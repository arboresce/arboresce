use arboresce_application::{MigrationRecord, ReadFailure, WriteFailure, WriteOutcome};
use arboresce_domain::{CalendarDate, ContentRevision, Identifier};

use std::{
    future::Future,
    task::{Context, Poll, Waker},
};

use serde_json::{Value, json};

mod support {
    pub mod authority;
    pub mod bootstrap;
    pub mod principal;
}

#[test]
fn complete_independent_record_literals() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/integration/postgres/fixtures/migration-harness-v1/cases.json"
    ))
    .unwrap();
    let records = fixture["records"].as_object().unwrap();
    assert_eq!(records.len(), 4);
    for (name, row) in records {
        let organization: Identifier = row["organization_id"].as_str().unwrap().parse().unwrap();
        let id: Identifier = row["record_id"].as_str().unwrap().parse().unwrap();
        let revision: ContentRevision = row["content_revision"].as_str().unwrap().parse().unwrap();
        let date: CalendarDate = row["date"].as_str().unwrap().parse().unwrap();
        let record = MigrationRecord::new(organization, id, revision, date);
        assert_eq!(record.organization().to_string(), row["organization_id"]);
        assert_eq!(record.record().to_string(), row["record_id"]);
        assert_eq!(record.revision().to_string(), row["content_revision"]);
        assert_eq!(record.date().to_string(), row["date"]);
        let preserved = record;
        assert_eq!(preserved, record);
        println!(
            "collector case=application-record-{name} pid={}",
            std::process::id()
        );
    }
}

#[test]
fn outcome_types_preserve_distinct_uncertainty_and_failures() {
    fn transfer_traits<T: Send + Sync + Copy + Eq>() {}
    transfer_traits::<MigrationRecord>();
    transfer_traits::<WriteOutcome>();
    transfer_traits::<ReadFailure>();
    transfer_traits::<arboresce_application::policy::principal::IdentityKey>();
    transfer_traits::<arboresce_application::policy::principal::StoredPrincipal>();
    transfer_traits::<arboresce_application::policy::principal::PrincipalMatches>();
    transfer_traits::<arboresce_application::policy::principal::PrincipalFailure>();
    transfer_traits::<arboresce_application::policy::principal::DirectoryFailure>();
    assert_ne!(
        WriteOutcome::Unknown,
        WriteOutcome::NotCommitted(WriteFailure::Unavailable)
    );
    assert_ne!(
        WriteOutcome::NotCommitted(WriteFailure::Conflict),
        WriteOutcome::NotCommitted(WriteFailure::Rejected)
    );
    assert_ne!(ReadFailure::Unavailable, ReadFailure::InvalidStoredData);
}

#[test]
fn complete_original_principal_mapping() {
    let data = include_str!("../../../tests/security/fixtures/authority-v1/cases.json");
    assert!(data.len() <= 8_388_608);
    let authority: Value = serde_json::from_str(data).unwrap();
    let data = include_str!("../../../tests/security/fixtures/principal-directory-v1/cases.json");
    assert!(data.len() <= 65_536);
    let fixture: Value = serde_json::from_str(data).unwrap();
    let cases: Vec<_> = authority["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["group"] == "principal_mapping")
        .collect();
    assert_eq!(cases.len(), 5);
    println!();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let oracle = fixture["original_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap();
        let projection = json!({
            "trusted_key_control_observation": case["input"]["trusted_key_control_observation"],
            "canonical_matching_principal_ids": case["input"]["canonical_matching_principal_ids"],
        });
        let probe = support::principal::DirectoryProbe::new(projection.clone(), true);
        let mut future = std::pin::pin!(probe.map());
        let mut context = Context::from_waker(Waker::noop());
        let first = future.as_mut().poll(&mut context);
        let result = match first {
            Poll::Ready(value) => value,
            Poll::Pending => match future.as_mut().poll(&mut context) {
                Poll::Ready(value) => value,
                Poll::Pending => panic!("directory future did not complete"),
            },
        };
        let observed = probe.observation(result);
        assert_eq!(observed, oracle["expected"], "{id}");
        assert_eq!(probe.input_after(), projection);
        assert_eq!(observed["principal_id"], case["expected"]["principal_id"]);
        assert_eq!(
            if result.is_ok() { "admit" } else { "deny" },
            case["expected"]["decision"]
        );
        let reason = match observed["failure"].as_str() {
            None => "exact_canonical_mapping",
            Some("UnknownCanonicalMapping") => "unknown_canonical_mapping",
            Some("UnsupportedIdentityProfile") => "unsupported_identity_profile",
            Some("CanonicalUniquenessViolation") => "canonical_uniqueness_violation",
            _ => panic!("unexpected original principal failure"),
        };
        assert_eq!(reason, case["expected"]["reason"]);
        assert_eq!(case["expected"]["new_principals_created"], json!([]));
        assert_eq!(case["expected"]["human_assurance_issued"], false);
        assert_eq!(case["expected"]["cryptographic_verification_claim"], false);
        println!("collector case={id} pid={}", std::process::id());
    }
}

#[test]
fn complete_principal_directory_port() {
    let data = include_str!("../../../tests/security/fixtures/principal-directory-v1/cases.json");
    assert!(data.len() <= 65_536);
    let fixture: Value = serde_json::from_str(data).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 31);
    println!();
    for case in cases {
        let projection =
            json!({"identity":case["input"]["identity"], "directory":case["input"]["directory"]});
        let probe = support::principal::DirectoryProbe::new(projection.clone(), false);
        let mut future = std::pin::pin!(probe.map());
        let mut context = Context::from_waker(Waker::noop());
        let first = future.as_mut().poll(&mut context);
        let result = match first {
            Poll::Ready(value) => value,
            Poll::Pending => match future.as_mut().poll(&mut context) {
                Poll::Ready(value) => value,
                Poll::Pending => panic!("directory future did not complete"),
            },
        };
        assert_eq!(
            probe.observation(result),
            case["expected"],
            "{}",
            case["id"]
        );
        assert_eq!(probe.input_after(), projection);
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_checked_stored_principals() {
    let data = include_str!("../../../tests/security/fixtures/principal-directory-v1/cases.json");
    assert!(data.len() <= 65_536);
    let fixture: Value = serde_json::from_str(data).unwrap();
    let cases = fixture["stored_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    println!();
    for case in cases {
        assert_eq!(
            support::principal::stored_observation(&case["input"]),
            case["expected"],
            "{}",
            case["id"]
        );
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_original_current_authority() {
    let raw = include_str!("../../../tests/security/fixtures/authority-v1/cases.json");
    assert!(raw.len() <= 8_388_608);
    let original: Value = serde_json::from_str(raw).unwrap();
    let raw = include_str!("../../../tests/security/fixtures/canonical-authority-v1/cases.json");
    assert!(raw.len() <= 8_388_608);
    let fixture: Value = serde_json::from_str(raw).unwrap();
    let cases: Vec<_> = original["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["group"] == "current_authority")
        .collect();
    assert_eq!(cases.len(), 72);
    assert_eq!(fixture["original_case_ids"].as_array().unwrap().len(), 72);
    let literals = fixture["original_admitted_observations"]
        .as_array()
        .unwrap();
    assert_eq!(literals.len(), 9);
    println!();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        assert!(
            fixture["original_case_ids"]
                .as_array()
                .unwrap()
                .contains(&case["id"])
        );
        let mut runtime = json!({
            "canonical_snapshot":case["input"]["canonical_snapshot"],
            "request":case["input"]["request"],
            "configuration":{
                "supported_policies":["authority_fixture_v1"],
                "supported_source_policies":["source_current"],
            },
        });
        let before = runtime.clone();
        let initial = support::authority::observe(&runtime);
        assert_eq!(runtime, before);
        let observed = if !case["input"]["final_effect_snapshot"].is_null() {
            assert!(
                initial["failure"].is_null(),
                "initial decision {id}: {initial}"
            );
            runtime["canonical_snapshot"] = case["input"]["final_effect_snapshot"].clone();
            let before = runtime.clone();
            let observed = support::authority::observe(&runtime);
            assert_eq!(runtime, before);
            observed
        } else {
            initial
        };
        assert_eq!(
            if observed["failure"].is_null() {
                "admit"
            } else {
                "deny"
            },
            case["expected"]["decision"],
            "{id}: {observed}"
        );
        if observed["failure"].is_null() {
            let expected = literals.iter().find(|r| r["id"] == id).unwrap();
            assert_eq!(observed["observation"], expected["expected"], "{id}");
            assert_eq!(
                observed["observation"]["executor_principal_id"],
                case["expected"]["actual_executor_principal_id"]
            );
            assert_eq!(
                observed["observation"]["selected_chain_id"],
                case["expected"]["selected_chain_id"]
            );
            assert_eq!(
                case["expected"]["visible_resource_ids"],
                json!([observed["observation"]["resource"]["id"]])
            );
        } else {
            assert!(observed["observation"].is_null());
            assert_eq!(case["expected"]["visible_resource_ids"], json!([]));
        }
        // This immutable pure API enables no effects or human mechanism.
        assert_eq!(case["expected"]["effects"], json!([]));
        assert_eq!(case["expected"]["human_assurance_issued"], false);
        println!("collector case={id} pid={}", std::process::id());
    }
}

#[test]
fn complete_canonical_authority_profile() {
    let raw = include_str!("../../../tests/security/fixtures/canonical-authority-v1/cases.json");
    assert!(raw.len() <= 8_388_608);
    let fixture: Value = serde_json::from_str(raw).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 119);
    println!();
    for case in cases {
        let runtime = json!({
            "canonical_snapshot":case["input"]["canonical_snapshot"],
            "request":case["input"]["request"],
            "configuration":case["input"]["configuration"],
        });
        let before = runtime.clone();
        let observed = support::authority::observe(&runtime);
        assert_eq!(runtime, before);
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_canonical_authority_failures() {
    let raw = include_str!("../../../tests/security/fixtures/canonical-authority-v1/cases.json");
    assert!(raw.len() <= 8_388_608);
    let fixture: Value = serde_json::from_str(raw).unwrap();
    let cases = fixture["error_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 6);
    println!();
    for case in cases {
        assert_eq!(
            support::authority::error_observation(&case["input"]),
            case["expected"],
            "{}",
            case["id"]
        );
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_principal_failure_interoperability() {
    let data = include_str!("../../../tests/security/fixtures/principal-directory-v1/cases.json");
    assert!(data.len() <= 65_536);
    let fixture: Value = serde_json::from_str(data).unwrap();
    let cases = fixture["error_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    println!();
    for case in cases {
        assert_eq!(
            support::principal::error_observation(&case["input"]),
            case["expected"],
            "{}",
            case["id"]
        );
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_original_bootstrap_scope() {
    let raw = include_str!("../../../tests/security/fixtures/authority-v1/cases.json");
    assert!(raw.len() <= 8_388_608);
    let original: Value = serde_json::from_str(raw).unwrap();
    let raw = include_str!("../../../tests/security/fixtures/bootstrap-scope-v1/profile.json");
    assert!(raw.len() <= 65_536);
    let profile: Value = serde_json::from_str(raw).unwrap();
    let cases: Vec<_> = original["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["group"] == "bootstrap_scope")
        .collect();
    assert_eq!(cases.len(), 10);
    println!();
    for case in cases {
        let before = case["input"].clone();
        let actual = support::bootstrap::observe(&case["input"], &profile["fixture_configuration"]);
        assert_eq!(case["input"], before);
        assert_eq!(actual["scope"], case["expected"], "{}", case["id"]);
        assert_eq!(case["input"]["writer_enabled"], false);
        assert_eq!(case["input"]["independent_lifecycle_proof"], "not_supplied");
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_bounded_bootstrap_scope_profile() {
    let raw = include_str!("../../../tests/security/fixtures/bootstrap-scope-v1/cases.json");
    assert!(raw.len() <= 131_072);
    let fixture: Value = serde_json::from_str(raw).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 33);
    println!();
    for case in cases {
        let before = case["input"].clone();
        let actual = support::bootstrap::observe(&case["input"], &case["input"]["configuration"]);
        assert_eq!(case["input"], before);
        assert_eq!(actual, case["expected"], "{}", case["id"]);
        assert_eq!(case["input"]["writer_enabled"], false);
        assert_eq!(case["input"]["independent_lifecycle_proof"], "not_supplied");
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}
