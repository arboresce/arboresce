use std::{error::Error, fmt};

use proc_macro2::{TokenStream, TokenTree};
use syn::{
    UseTree,
    visit::{self, Visit},
};

const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;

fn word(ident: &proc_macro2::Ident) -> String {
    let spelling = ident.to_string();
    spelling.strip_prefix("r#").unwrap_or(&spelling).to_owned()
}

fn panic_name(name: &str) -> bool {
    matches!(
        name,
        "unwrap"
            | "expect"
            | "unwrap_err"
            | "expect_err"
            | "panic"
            | "panic_any"
            | "resume_unwind"
            | "unreachable"
            | "todo"
            | "unimplemented"
    )
}

fn explicit_path_attribute(stream: TokenStream) -> bool {
    let tokens: Vec<_> = stream.into_iter().collect();
    tokens.iter().enumerate().any(|(index, token)| match token {
        TokenTree::Group(group) => explicit_path_attribute(group.stream()),
        TokenTree::Ident(ident) => {
            word(ident) == "path"
                && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '=')
        }
        _ => false,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Layer {
    Domain,
    Application,
    Workflow,
    Adapter,
    Composition,
    Tool,
    Proof,
}

impl Layer {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "domain" => Self::Domain,
            "application" => Self::Application,
            "workflow" => Self::Workflow,
            "adapter" => Self::Adapter,
            "composition" => Self::Composition,
            "tool" => Self::Tool,
            "proof" => Self::Proof,
            _ => return None,
        })
    }

    fn pure(self) -> bool {
        matches!(self, Self::Domain | Self::Application | Self::Workflow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardError {
    InvalidInput,
    Dependency,
    Source,
}

impl fmt::Display for GuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "invalid verification input",
            Self::Dependency => "unqualified architecture dependency",
            Self::Source => "forbidden architecture source",
        })
    }
}
impl Error for GuardError {}

/// This is a closed initial profile. New pure dependencies need qualification.
pub fn check_dependency(
    layer: Layer,
    package: &str,
    version: &str,
    kind: &str,
) -> Result<(), GuardError> {
    match kind {
        "dev" => return Ok(()),
        "normal" | "build" => {}
        _ => return Err(GuardError::InvalidInput),
    }
    let allowed = match layer {
        Layer::Domain => false,
        Layer::Application => package == "arboresce-domain" && version == "0.0.0",
        Layer::Workflow => match package {
            "arboresce-domain" | "arboresce-application" => version == "0.0.0",
            "temporalio-sdk" => version == "1.0.0",
            _ => false,
        },
        Layer::Adapter | Layer::Composition | Layer::Tool | Layer::Proof => true,
    };
    if allowed {
        Ok(())
    } else {
        Err(GuardError::Dependency)
    }
}

/// Check syntax and the configured explicit import/path boundaries.
pub fn check_source(layer: Layer, source: &str) -> Result<(), GuardError> {
    check_source_selected(layer, source, false)
}

/// Selected by the owning complete workspace graph, never a source filename.
/// This admits in-memory writer types, not ambient I/O or general macro imports.
pub(crate) fn check_source_selected(
    layer: Layer,
    source: &str,
    pure_writer: bool,
) -> Result<(), GuardError> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(GuardError::InvalidInput);
    }
    let file = syn::parse_file(source).map_err(|_| GuardError::InvalidInput)?;
    let mut guard = SourceGuard {
        layer,
        pure_writer: pure_writer && layer == Layer::Domain,
        denied: false,
    };
    guard.visit_file(&file);
    if guard.denied {
        Err(GuardError::Source)
    } else {
        Ok(())
    }
}

struct SourceGuard {
    layer: Layer,
    pure_writer: bool,
    denied: bool,
}

impl SourceGuard {
    fn path(&mut self, parts: &[String], importing: bool, glob: bool) {
        let Some(first) = parts.first().map(String::as_str) else {
            return;
        };
        self.denied |= parts.last().is_some_and(|name| panic_name(name))
            || (importing && parts.last().is_some_and(|name| name == "include"))
            || (matches!(first, "std" | "core")
                && parts.get(1).is_some_and(|p| p == "panic")
                && glob);
        if !self.layer.pure() {
            return;
        }
        let second = parts.get(1).map(String::as_str);
        let writer_type = self.pure_writer
            && !glob
            && first == "std"
            && second == Some("io")
            && ((parts.len() == 3
                && matches!(parts[2].as_str(), "Write" | "Result" | "ErrorKind"))
                || (parts.len() == 4 && parts[2] == "ErrorKind" && parts[3] == "FileTooLarge"));
        let denied = match first {
            "std" | "core" => {
                (importing && parts.len() == 1)
                    || matches!(
                        second,
                        Some("fs" | "net" | "env" | "process" | "thread" | "random")
                    )
                    || (second == Some("io") && !writer_type)
                    || (second == Some("os")
                        && !(parts.get(2).is_some_and(|p| p == "raw")
                            || (parts
                                .get(2)
                                .is_some_and(|p| matches!(p.as_str(), "unix" | "windows"))
                                && parts.get(3).is_some_and(|p| p == "ffi"))))
                    || (second == Some("time")
                        && (parts.len() == 2
                            || glob
                            || parts
                                .iter()
                                .any(|p| matches!(p.as_str(), "SystemTime" | "Instant" | "self"))))
            }
            "rand" | "getrandom" | "reqwest" | "sqlx" | "tokio" | "axum" | "aws_sdk_s3" => true,
            "temporalio_sdk" if self.layer == Layer::Workflow => {
                glob || !matches!(
                    second,
                    Some(
                        "WorkflowContext"
                            | "WorkflowContextView"
                            | "WorkflowResult"
                            | "WorkflowTermination"
                            | "SyncWorkflowContext"
                            | "TimerOptions"
                            | "ActivityOptions"
                            | "ActivityCloseTimeouts"
                            | "LocalActivityOptions"
                            | "ChildWorkflowOptions"
                            | "CancellableFuture"
                            | "Signal"
                            | "workflows"
                    )
                )
            }
            name if name.starts_with("temporalio_") => true,
            name if name.starts_with("arboresce_") => {
                (matches!(self.layer, Layer::Domain | Layer::Application)
                    && name != "arboresce_domain")
                    || (self.layer == Layer::Workflow
                        && !matches!(name, "arboresce_domain" | "arboresce_application"))
            }
            _ => false,
        };
        self.denied |= denied;
    }

    fn import_leaf(&mut self, ident: &proc_macro2::Ident, prefix: &mut Vec<String>) {
        let name = word(ident);
        if name == "self" {
            self.path(prefix, true, false);
        } else {
            prefix.push(name);
            self.path(prefix, true, false);
            prefix.pop();
        }
    }

    fn imports(&mut self, tree: &UseTree, prefix: &mut Vec<String>) {
        match tree {
            UseTree::Path(p) => {
                prefix.push(word(&p.ident));
                self.imports(&p.tree, prefix);
                prefix.pop();
            }
            UseTree::Name(n) => self.import_leaf(&n.ident, prefix),
            UseTree::Rename(n) => self.import_leaf(&n.ident, prefix),
            UseTree::Glob(_) => self.path(prefix, true, true),
            UseTree::Group(g) => {
                for item in &g.items {
                    self.imports(item, prefix);
                }
            }
        }
    }

    fn tokens(&mut self, stream: TokenStream) {
        let tokens: Vec<_> = stream.into_iter().collect();
        let mut index = 0;
        while let Some(token) = tokens.get(index) {
            match token {
                TokenTree::Group(group) => self.tokens(group.stream()),
                TokenTree::Ident(ident) => {
                    let spelling = word(ident);
                    self.denied |= matches!(spelling.as_str(), "unsafe" | "include")
                        || (self.pure_writer && matches!(spelling.as_str(), "use" | "extern"))
                        || panic_name(&spelling)
                        || (self.layer.pure()
                            && matches!(
                                spelling.as_str(),
                                "print" | "println" | "eprint" | "eprintln"
                            ));
                    let mut parts = vec![spelling];
                    let mut next = index + 1;
                    while matches!(tokens.get(next), Some(TokenTree::Punct(p)) if p.as_char() == ':')
                        && matches!(tokens.get(next + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':')
                    {
                        let Some(TokenTree::Ident(part)) = tokens.get(next + 2) else {
                            break;
                        };
                        parts.push(word(part));
                        next += 3;
                    }
                    self.path(&parts, false, false);
                    index = next;
                    continue;
                }
                TokenTree::Literal(_) | TokenTree::Punct(_) => {}
            }
            index += 1;
        }
    }
}

impl<'ast> Visit<'ast> for SourceGuard {
    fn visit_expr_path(&mut self, n: &'ast syn::ExprPath) {
        // Syn stores a UFCS trait method in one path. Admit only the frozen
        // Write::write_all form, while still visiting self and all arguments.
        if self.pure_writer
            && let Some(qualified) = &n.qself
            && qualified.position == 3
            && n.path.segments.len() == 4
            && n.path.segments.iter().map(|s| word(&s.ident)).eq([
                "std",
                "io",
                "Write",
                "write_all",
            ])
        {
            for attribute in &n.attrs {
                self.visit_attribute(attribute);
            }
            self.visit_qself(qualified);
            for segment in &n.path.segments {
                self.visit_path_arguments(&segment.arguments);
            }
            return;
        }
        visit::visit_expr_path(self, n);
    }

    fn visit_item_use(&mut self, n: &'ast syn::ItemUse) {
        self.imports(&n.tree, &mut Vec::new());
    }
    fn visit_item_extern_crate(&mut self, n: &'ast syn::ItemExternCrate) {
        self.path(&[word(&n.ident)], true, false);
    }
    fn visit_path(&mut self, n: &'ast syn::Path) {
        self.path(
            &n.segments
                .iter()
                .map(|p| word(&p.ident))
                .collect::<Vec<_>>(),
            false,
            false,
        );
        visit::visit_path(self, n);
    }
    fn visit_macro(&mut self, n: &'ast syn::Macro) {
        self.visit_path(&n.path);
        self.denied |= n.path.segments.last().is_some_and(|p| {
            let name = word(&p.ident);
            name == "include"
                || (self.layer.pure()
                    && matches!(name.as_str(), "print" | "println" | "eprint" | "eprintln"))
        });
        self.tokens(n.tokens.clone());
    }
    fn visit_expr_method_call(&mut self, n: &'ast syn::ExprMethodCall) {
        self.denied |= panic_name(&word(&n.method));
        visit::visit_expr_method_call(self, n);
    }
    fn visit_attribute(&mut self, n: &'ast syn::Attribute) {
        self.denied |= n
            .path()
            .segments
            .last()
            .is_some_and(|p| word(&p.ident) == "path");
        if n.path()
            .segments
            .last()
            .is_some_and(|p| word(&p.ident) == "cfg_attr")
            && let syn::Meta::List(list) = &n.meta
        {
            self.denied |= explicit_path_attribute(list.tokens.clone());
        }
        visit::visit_attribute(self, n);
    }
    fn visit_expr_unsafe(&mut self, n: &'ast syn::ExprUnsafe) {
        self.denied = true;
        visit::visit_expr_unsafe(self, n);
    }
    fn visit_signature(&mut self, n: &'ast syn::Signature) {
        self.denied |= n.unsafety.is_some();
        visit::visit_signature(self, n);
    }
    fn visit_item_impl(&mut self, n: &'ast syn::ItemImpl) {
        self.denied |= n.unsafety.is_some();
        visit::visit_item_impl(self, n);
    }
    fn visit_item_trait(&mut self, n: &'ast syn::ItemTrait) {
        self.denied |= n.unsafety.is_some();
        visit::visit_item_trait(self, n);
    }
    fn visit_item_foreign_mod(&mut self, n: &'ast syn::ItemForeignMod) {
        self.denied |= n.unsafety.is_some();
        visit::visit_item_foreign_mod(self, n);
    }
}
