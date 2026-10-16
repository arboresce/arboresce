//! Measured input-only directory producer and native API observations.

use std::{
    error::Error,
    future::poll_fn,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::Poll,
};

use arboresce_application::policy::principal::{
    DirectoryFailure, IdentityKey, PrincipalDirectory, PrincipalFailure, PrincipalMatches,
    StoredPrincipal, map_principal,
};
use arboresce_domain::Identifier;
use serde_json::{Value, json};

pub struct DirectoryProbe {
    input: Value,
    original: bool,
    calls: Mutex<(usize, Option<IdentityKey>)>,
    polls: AtomicUsize,
}

impl DirectoryProbe {
    pub fn new(input: Value, original: bool) -> Self {
        Self {
            input,
            original,
            calls: Mutex::new((0, None)),
            polls: AtomicUsize::new(0),
        }
    }

    pub async fn map(&self) -> Result<Identifier, PrincipalFailure> {
        let input = if self.original {
            &self.input["trusted_key_control_observation"]
        } else {
            &self.input["identity"]
        };
        let identity = IdentityKey::parse(
            input["profile"].as_str().unwrap(),
            input["version"].as_str().unwrap(),
            input["public_key"].as_str().unwrap(),
        )?;
        map_principal(self, identity).await
    }

    pub fn input_after(&self) -> Value {
        self.input.clone()
    }

    pub fn observation(&self, result: Result<Identifier, PrincipalFailure>) -> Value {
        let calls = self.calls.lock().unwrap();
        let (principal, failure) = match result {
            Ok(value) => (Some(value.to_string()), None),
            Err(error) => (None, Some(error.to_string())),
        };
        json!({
            "principal_id": principal,
            "failure": failure,
            "lookup_calls": calls.0,
            "query": calls.1.map(identity_observation),
            "directory_polls": self.polls.load(Ordering::Relaxed),
        })
    }

    fn response(&self) -> Result<PrincipalMatches, DirectoryFailure> {
        if self.original {
            let rows = self.input["canonical_matching_principal_ids"]
                .as_array()
                .unwrap();
            assert!(rows.len() <= 2);
            return match rows.as_slice() {
                [] => Ok(PrincipalMatches::Missing),
                [principal] => {
                    let identity = &self.input["trusted_key_control_observation"];
                    StoredPrincipal::decode(
                        identity["profile"].as_str().unwrap(),
                        identity["version"].as_str().unwrap(),
                        identity["public_key"].as_str().unwrap(),
                        principal.as_str().unwrap(),
                    )
                    .map(PrincipalMatches::One)
                }
                _ => Ok(PrincipalMatches::Multiple),
            };
        }
        let input = &self.input["directory"];
        match input["outcome"].as_str().unwrap() {
            "unavailable" => Err(DirectoryFailure::Unavailable),
            "invalid_stored_data" => Err(DirectoryFailure::InvalidStoredData),
            "rows" => {
                let rows = input["rows"].as_array().unwrap();
                assert!(rows.len() <= 2);
                match rows.as_slice() {
                    [] => Ok(PrincipalMatches::Missing),
                    [row] => decode(row).map(PrincipalMatches::One),
                    _ => Ok(PrincipalMatches::Multiple),
                }
            }
            _ => panic!("unadmitted directory fixture outcome"),
        }
    }
}

impl PrincipalDirectory for DirectoryProbe {
    fn lookup(
        &self,
        identity: IdentityKey,
    ) -> impl std::future::Future<Output = Result<PrincipalMatches, DirectoryFailure>> + Send {
        {
            let mut calls = self.calls.lock().unwrap();
            calls.0 += 1;
            calls.1 = Some(identity);
        }
        let result = self.response();
        let mut pending = self.input["directory"]["pending_first"]
            .as_bool()
            .unwrap_or(false);
        poll_fn(move |context| {
            self.polls.fetch_add(1, Ordering::Relaxed);
            if pending {
                pending = false;
                context.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(result)
            }
        })
    }
}

fn identity_observation(identity: IdentityKey) -> Value {
    json!({
        "profile": identity.profile(),
        "version": identity.version(),
        "public_key_bytes": identity.public_key(),
    })
}

fn decode(input: &Value) -> Result<StoredPrincipal, DirectoryFailure> {
    StoredPrincipal::decode(
        input["profile"].as_str().unwrap(),
        input["version"].as_str().unwrap(),
        input["public_key"].as_str().unwrap(),
        input["principal_id"].as_str().unwrap(),
    )
}

pub fn stored_observation(input: &Value) -> Value {
    match decode(input) {
        Ok(record) => json!({
            "failure": null,
            "principal_id": record.principal().to_string(),
            "identity": identity_observation(record.identity()),
        }),
        Err(error) => json!({
            "failure": error.to_string(),
            "principal_id": null,
            "identity": null,
        }),
    }
}

struct FailingFormatter;

impl std::fmt::Write for FailingFormatter {
    fn write_str(&mut self, _value: &str) -> std::fmt::Result {
        Err(std::fmt::Error)
    }
}

fn diagnostic<E: Error + Copy + Eq + 'static>(error: E) -> Value {
    let boxed: Box<dyn Error> = Box::new(error);
    let formatting = std::fmt::write(&mut FailingFormatter, format_args!("{error}"));
    json!({
        "display": boxed.to_string(),
        "source_is_none": boxed.source().is_none(),
        "boxed_variant_preserved": boxed.downcast_ref::<E>() == Some(&error),
        "formatter_failure_propagated": formatting.is_err(),
    })
}

pub fn error_observation(input: &Value) -> Value {
    let variant = input["variant"].as_str().unwrap();
    if input["kind"] == "principal" {
        let examples = [
            (
                "UnsupportedIdentityProfile",
                PrincipalFailure::UnsupportedIdentityProfile,
            ),
            ("InvalidPublicKey", PrincipalFailure::InvalidPublicKey),
            (
                "UnknownCanonicalMapping",
                PrincipalFailure::UnknownCanonicalMapping,
            ),
            (
                "CanonicalUniquenessViolation",
                PrincipalFailure::CanonicalUniquenessViolation,
            ),
            ("Unavailable", PrincipalFailure::Unavailable),
            ("InvalidStoredData", PrincipalFailure::InvalidStoredData),
        ];
        diagnostic(
            examples
                .iter()
                .find(|(name, _)| *name == variant)
                .unwrap()
                .1,
        )
    } else {
        assert_eq!(input["kind"], "directory");
        let examples = [
            ("Unavailable", DirectoryFailure::Unavailable),
            ("InvalidStoredData", DirectoryFailure::InvalidStoredData),
        ];
        diagnostic(
            examples
                .iter()
                .find(|(name, _)| *name == variant)
                .unwrap()
                .1,
        )
    }
}
