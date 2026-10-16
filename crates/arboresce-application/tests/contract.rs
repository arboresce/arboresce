use arboresce_application::{MigrationRecord, ReadFailure, WriteFailure, WriteOutcome};
use arboresce_domain::{CalendarDate, ContentRevision, Identifier};

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
