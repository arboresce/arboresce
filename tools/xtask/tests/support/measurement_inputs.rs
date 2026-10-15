//! Measured preparation for independently frozen native receipt inputs.
use super::PreparedRoot;
use serde_json::Value;
use std::{
    ffi::OsString,
    fs, io,
    os::unix::{
        ffi::OsStringExt,
        fs::{PermissionsExt, symlink},
    },
    path::PathBuf,
    process::Command,
};

pub struct Input {
    pub owner: PreparedRoot,
    pub root: PathBuf,
}

impl Input {
    pub fn new(base: &Value, case: &Value) -> io::Result<Self> {
        let owner = PreparedRoot::new()?;
        let mut root = owner.path.join("measurement");
        fs::create_dir(&root)?;
        fs::create_dir(root.join("launches"))?;
        fs::create_dir(root.join("exits"))?;
        let mut context = base.clone();
        if let Some(set) = case["set"].as_object() {
            context.as_object_mut().unwrap().extend(set.clone());
        }
        if let Some(key) = case["remove"].as_str() {
            context.as_object_mut().unwrap().remove(key);
        }
        let mut bytes = case["raw"]
            .as_str()
            .map_or_else(|| context.to_string(), str::to_owned)
            .into_bytes();
        if let Some(count) = case["context_padding_bytes"].as_u64() {
            bytes.resize(count as usize, b' ');
        }
        fs::write(root.join("context.json"), bytes)?;
        match case["prepare"].as_str().unwrap_or("") {
            "missing-root" => fs::remove_dir_all(&root)?,
            "root-file" => {
                fs::remove_dir_all(&root)?;
                fs::write(&root, b"file")?;
            }
            "root-link" => {
                let link = owner.path.join("root-link");
                symlink(&root, &link)?;
                root = link;
            }
            "missing-context" => fs::remove_file(root.join("context.json"))?,
            "context-directory" => {
                fs::remove_file(root.join("context.json"))?;
                fs::create_dir(root.join("context.json"))?;
            }
            "context-link" => {
                let target = owner.path.join("context-target");
                fs::rename(root.join("context.json"), &target)?;
                symlink(target, root.join("context.json"))?;
            }
            "missing-launches" | "missing-exits" => {
                let name = case["prepare"]
                    .as_str()
                    .unwrap()
                    .strip_prefix("missing-")
                    .unwrap();
                fs::remove_dir(root.join(name))?;
            }
            "launches-file" | "exits-file" | "launches-link" | "exits-link" => {
                let recipe = case["prepare"].as_str().unwrap();
                let (name, mode) = recipe.split_once('-').unwrap();
                fs::remove_dir(root.join(name))?;
                if mode == "file" {
                    fs::write(root.join(name), b"file")?;
                } else {
                    let target = owner.path.join("directory-target");
                    fs::create_dir(&target)?;
                    symlink(target, root.join(name))?;
                }
            }
            "" | "context-oversized" => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unknown measurement preparation",
                ));
            }
        }
        Ok(Self { owner, root })
    }

    pub fn command(&self, case: &Value) -> io::Result<Command> {
        let mode = case["program_mode"].as_str().unwrap_or("");
        let mut program = OsString::from(case["program"].as_str().unwrap_or("/usr/bin/true"));
        let mut cwd = self.owner.path.clone();
        if case["cwd"] == "subdir" || mode == "relative" {
            cwd.push("subdir");
            fs::create_dir(&cwd)?;
        } else if case["cwd"] == "missing" {
            cwd.push("missing");
        }
        match mode {
            "relative" => {
                symlink("/usr/bin/true", cwd.join("tool"))?;
                program = "./tool".into();
            }
            "missing" => program = self.owner.path.join("missing-tool").into_os_string(),
            "directory" => {
                let path = self.owner.path.join("tool-directory");
                fs::create_dir(&path)?;
                program = path.into_os_string();
            }
            "not-executable" | "oversized" | "exact-limit" => {
                let path = self.owner.path.join("tool-file");
                let file = fs::File::create_new(&path)?;
                let count = match mode {
                    "oversized" => 67108865,
                    "exact-limit" => 67108864,
                    _ => 0,
                };
                file.set_len(count)?;
                fs::set_permissions(
                    &path,
                    fs::Permissions::from_mode(if mode == "not-executable" {
                        0o644
                    } else {
                        0o755
                    }),
                )?;
                program = path.into_os_string();
            }
            "path" | "path-link" | "path-empty" | "path-removed" | "path-access-failure" => {
                program = "tool".into()
            }
            "" => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unknown measurement executable",
                ));
            }
        }
        let mut command = Command::new(program);
        command.current_dir(cwd);
        match mode {
            "path" | "path-link" => {
                let bin = self.owner.path.join("bin");
                fs::create_dir(&bin)?;
                symlink("/usr/bin/true", bin.join("tool"))?;
                let selected = if mode == "path-link" {
                    let link = self.owner.path.join("bin-link");
                    symlink(&bin, &link)?;
                    link
                } else {
                    bin
                };
                command.env("PATH", selected);
            }
            "path-access-failure" => {
                let mut paths = Vec::new();
                for (name, key) in [("first-bin", "first_mode"), ("second-bin", "second_mode")] {
                    let bin = self.owner.path.join(name);
                    fs::create_dir(&bin)?;
                    fs::copy("/usr/bin/true", bin.join("tool"))?;
                    fs::set_permissions(
                        bin.join("tool"),
                        fs::Permissions::from_mode(case[key].as_u64().unwrap() as u32),
                    )?;
                    paths.push(bin);
                }
                command.env(
                    "PATH",
                    std::env::join_paths(paths).map_err(io::Error::other)?,
                );
            }
            "path-empty" => {
                command.env("PATH", "");
            }
            "path-removed" => {
                command.env_remove("PATH");
            }
            _ => {}
        }
        if let Some(arguments) = case["arguments"].as_array() {
            for argument in arguments {
                command.arg(argument.as_str().unwrap());
            }
        }
        if let Some(bytes) = case["argument_bytes"].as_array() {
            command.arg(OsString::from_vec(
                bytes.iter().map(|n| n.as_u64().unwrap() as u8).collect(),
            ));
        }
        if let Some(key) = case["remove_environment"].as_str() {
            command.env_remove(key);
        }
        Ok(command)
    }
}

/// These fixture Commands have only default streams; copy their explicit configuration.
pub fn bind_selected(requested: &Command, selected: &std::path::Path) -> Command {
    let mut bound = Command::new(selected);
    bound.args(requested.get_args());
    if let Some(cwd) = requested.get_current_dir() {
        bound.current_dir(cwd);
    }
    for (key, value) in requested.get_envs() {
        if let Some(value) = value {
            bound.env(key, value);
        } else {
            bound.env_remove(key);
        }
    }
    bound
}
