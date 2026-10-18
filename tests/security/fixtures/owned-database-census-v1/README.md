# Owned database census deadline fixtures

These original public literals define three controlled observations of the
bounded census loop under the [resource](../../../../docs/requirements/resource-profiles.md),
[storage](../../../../docs/requirements/storage-and-search.md),
[security](../../../../docs/requirements/security-and-privacy.md) and
[testing](../../../../docs/requirements/testing-and-coverage.md) requirements.
They preserve the original one-second census and every existing native,
migration, authority, shared-admission and transport-close expectation.

Only case.input reaches the controlled producer. The empty common_inputs
object declares no selectable budget or service. IDs, expected objects, profile,
rights, source pins and counts remain outside runtime decisions. The caller
compares the complete actual semantic object after the producer returns.
Unexpected outer preparation, runtime, observation or error-class failures fail
the check.

The producer must share the actual census loop used by the exclusively owned
service. That service keeps its original deadline, compiled SQL, real count
provider, initial observed count zero, cooperative yield and typed failures.
The controlled facade selects only compiled futures, sample values and deadline
constructions. It accepts no path, PID, SQL, connection, cleanup target, arbitrary
clock or replacement expected result.

## Fixed scenarios

- ready_empty supplies one immediately ready zero sample before a fresh
  one-second deadline. The loop succeeds; one probe starts and completes.
- pending_query supplies a first probe that remains pending until the original
  one-second deadline. No sample has completed, so the retained observed count
  is zero. The actual Tokio timeout rejects the query. The pending future owns
  a destruction marker: it starts, never completes and is dropped by timeout.
- ready_busy_after_deadline supplies an immediately ready sample of one with
  a compiled already-expired deadline. The selected Tokio1.53.3
  [immediate-ready contract](https://docs.rs/tokio/1.53.3/tokio/time/fn.timeout_at.html)
  permits the ready value. The loop must then reject at its monotonic deadline
  check, retaining count one without a query-timeout error.

The pending scenario starts with no completed sample; it does not invent an
earlier busy result or depend on winning a race before the deadline. A fixed
future and its actual counters and destruction marker establish the observations.
No sleeps, timer suspension, host-clock changes or manufactured Elapsed value
may establish a case. There are no spawned tasks or external resources to
abandon. Record the actual returned error and timing as bounded evidence while
comparing the complete semantic object defined in cases.json.

These are controlled loop observations. The existing actual PostgreSQL cases
still require a real held backend, positive reset refusal before destructive
work, complete canonical and migration conservation, positive owned connection
close and a subsequent empty database census. Either original native census
refusal mechanism retains its meaning. A controlled count or future supplies
no native SQL, installed product, recovery, user or production qualification.

The fixed three-file package uses the existing bounded closed loader, with
compiled profile and cases versions. Its public source pins and README/cases
fingerprints must be validated before consumption. Existing fixture packages,
whole expected objects, dependencies, source roles, targets, test bodies,
deadlines, charged-slot rules and strict coverage requirements remain required.
Every exercised helper and process remains measured. Fresh owning and
independent standalone qualification and the original independent source,
expected-value and delivery reviews remain separate gates.

The literals use the repository's MIT OR Apache-2.0 license. No external data,
personal information, credentials or private provenance is included.
