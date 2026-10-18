# Additional shared admission input and native boundaries

These original public synthetic literals use MIT OR Apache-2.0. They supplement
the independently admitted [shared package](../shared-admission-v1/README.md)
without changing any of its 237 cases, the original authority 185 cases or their 23
shared scenarios. They contain no customer records, secrets, operational
credentials, provider calls or accepted product effects.

The owning requirements are [shared resources](../../../../docs/requirements/resource-profiles.md),
[relational enforcement](../../../../docs/requirements/storage-and-search.md),
[security](../../../../docs/requirements/security-and-privacy.md) and
[testing](../../../../docs/requirements/testing-and-coverage.md). Their exact
source pins, package bounds and runtime projection are frozen in profile.json.
A static literal does not qualify a real transaction, healthy clock, current
authority, authentication, financial assurance, recovery or production service.

## Input preparation

app_preparation has 41 complete malformed field inputs for the existing pure
claim, quota, lease, command and finite-error preparation boundaries. Each must
remain the exact finite preparation error, rather than become a product
decision. All original inputs and expected observations remain independent.

native_preparation has 31 direct typed caller/policy/claim/command and injected
Clock cases. Negative cases return {"accepted":false}; they perform no SQL.
Direct outcome-consuming commands require an explicit finite outcome token:
not_dispatched, unknown, accepted_running or completed. Missing, null or
wrong-type observations cannot become proof of no dispatch. A null stored
outcome remains valid where its independently declared type is optional.

For the original legacy projection only, a missing release observation is
explicitly UNKNOWN. This preserves its existing stale-fence and wrong-owner
cases, whose owner/fence mismatch is still denied before release. Do not invent
not_dispatched to fill absent evidence. This projection is input normalization,
not a new external receipt or an HTTP authority flag.

Clock sequence admission accepts one through three supplied timestamp-or-null
samples. The positive direct probe calls the actual injected Clock four times
over the declared three samples: the third value remains available after the
second null fails, and exhaustion fails too. Samples and calls are observed,
not selected by a case identity. This supplies no real clock-health evidence.

seed_preparation has 46 cases. Decode every supplied shared seed SQL bind value,
the declared authority-owner workspace and every finite row/allocation bound
BEFORE invoking the existing canonical reset/seed or any shared INSERT. The
original canonical snapshot and its existing validation remain required. Default
attempts_used is one only for state active, otherwise zero. Default admitted_at
and last_observed_at come from the explicit common inputs. The default next
reservation order is reservation count plus one. Row order is literal input
order, starting at one; an explicit next order does not rewrite those rows.
Optional supplied values are decoded rather than silently replaced.

The positive normalized ledger includes every limit, used counter, original
reservation field, explicit nullable dispatch outcome, attempts_used, admitted_at,
last_observed_at and reservation_order, plus the selected authority owner.
Observe the actual prepared typed values. Never echo an unvalidated input object
as a successful preparation result. The exact 128-row preparation control and
one-over 129 rejection concern typed/allocation bounds, not native persistence.
Signed-compatible resource inputs remain nonnegative; driver smallint binds
must fit signed 16-bit values.

Preparation does not duplicate or weaken SQL business constraints. It does not
infer authority or reject the deliberately declared SQL-valid cross-row counter
faults in the native cases. A preparation failure is distinct from an actual
database constraint, transport, transaction or cleanup failure.

race_preparation has 11 malformed permission-schedule inputs. Prepare the complete
typed caller, policy, claim, both injected clocks, selected owned resource and
canonical membership revision BEFORE acquiring the two physical sessions.
Missing selected resources or membership, invalid revision, fields or clocks
must fail preparation. The existing native permission schedules still supply
their separate successful real two-backend qualification.

## Actual SQL row channels

sql_decoders has 59 cases for the SAME typed budget, recognition and live-lease
decoders used by the production SQL readers. Refactor those readers to call the
shared decoding boundaries; do not introduce a test decoder or public writer.
The actual PostgreSQL driver must return one row through compiled parameterized
SQL. Bind the complete supplied object as serialized TEXT, converted inside fixed
`jsonb_to_record($1::text::jsonb)` column declarations. Preserve the existing SQLx
dependency features; no JSON feature or other feature expansion is admitted.
Do not interpolate supplied SQL or accept a supplied error result.

Budget global columns are active_limit, memory_limit, active_used, memory_used
and next_reservation_order as bigint, and last_observed_at as text. Organization
columns are pending_limit, active_limit, storage_limit, memory_limit, cost_limit,
pending_used, active_used, storage_used, memory_used and cost_used as bigint.
The complete positive observation contains both limit/used resource views,
observed_at and next_reservation_order.

Recognition columns are organization_id, reservation_id, principal_id, owner,
replica, total_deadline, admitted_at, last_observed_at and state as text; fence,
attempt_limit and attempts_used as bigint. Positive observations contain EVERY
typed recognition field, including all four finite states. Driver NULL, invalid
UUID or timestamp, negative integer and unknown state cases must produce the
actual decoder's InvalidStoredData.

Live columns are pending, active, storage_bytes, memory_bytes and cost_microusd
as bigint; expires_at and dispatch_outcome as text; outcome_present as boolean.
Decode the separate complete recognition row first. The positive live observation
contains every flattened typed claim/lease field, including admitted/observed
time, attempts, state and explicit nullable outcome. All four finite outcomes
and null remain distinct. NULL, unknown outcome, present-with-NULL, or a decoded
lease expiring before admission must fail the actual decoder.

The selected channel deliberately supplies malformed projection values without
altering canonical schema or pretending constraints admitted them. It is an
actual read-only SQL/driver/decoder boundary, not canonical storage, current
membership, a native authorization decision or product persistence. Unexpected
SQL, connection or cleanup errors must fail the probe; they cannot be translated
into the expected typed decoder error. Positives compare entire typed values,
and negatives the exact actual finite failure.

## Actual shared controls and schedules

native_controls has 20 cases on the selected actual prepared PostgreSQL service.
Each starts from its complete literal ledger and canonical seed. The controlled
lease cases operate on command, never a caller-supplied canonical lease.
Injected configuration and time retain their explicit synthetic qualification
scope. Every result is compared with its COMPLETE literal ledger, all four
physical shared tables, canonical authority and zero dispatch/effect counts.

Seven undercharged release cases independently cover five organization counters
and two global counters. SQL permits the declared nonnegative cross-row fault;
the real adapter must return InvalidStoredData and positively roll back every
temporary reservation, recognition and counter mutation. Retained charged
responsibility cannot disappear through partial subtraction or release.

Eight clock controls cover unavailable entry/after-lock/precommit samples,
backward observations, the stored observation boundary, exact expiry and exact
total deadline. Compare actual Clock calls and the complete unchanged state
after rejected transactions. A temporary renewal must roll back on a failed
final sample. The separate complete renew/release positives compare EVERY
resulting stored field, retained recognition and counter.

The unknown-reservation case observes NotFound without inventing a lease.
For the missing-live case, stored_fault specifies delete_live for one existing
reservation after the complete seed commits. Execute exactly one parameterized
owner DELETE, retain its actual affected-row count, then observe the faulted
state BEFORE invoking the adapter. InvalidStoredData and positive rollback must
preserve that complete faulted state; restore only through the next independently
declared reset. The fault never means the schema or decoder is replaced.

The allocator-collision case seeds the explicit next_reservation_order of one
while the existing first recognition retains ordinal one. The actual new claim
must encounter PostgreSQL unique violation 23505, become Conflict, and positively
roll back without partially charging counters or adding recognition. This is a
real database failure, not formatting a supplied error string.

native_races has TWO full real two-backend schedules. The sufficient-headroom
schedule admits both ordered claims after the actual conflicting global lock
wait, with exact ordinals, counters and complete stored state. It uses the same
independent factory/physical-backend/barrier requirements as the earlier
last-slot denial. The other schedule waits a real permission withdrawal behind a
claim whose final Clock sample fails. The claim rolls back completely, then the
permission change commits. Compare the entire uncharged seed ledger and actual
revoked authority, not just the error label.

race_guards has TWO explicitly supplied Future-poll controls for the same barrier
helper. A completed operation before its barrier is an error. A pending operation
allows the supplied ready barrier value. These are deterministic scheduling guard
observations ONLY; the supplied marker is never a SQL blocker or native lock proof.

## Consumer and qualification boundaries

Only common_inputs and case.input may reach preparation, adapters or native
helpers. Expected data, case IDs, rights, source pins and counts remain fixture
admission or thin-assertion metadata. Every complete expected object stays
independent from the implementation. No maintained generator is required or
provided; normal use reads the committed files.

The original 237 and 185/23 cases, all existing native and pure assertions, cohort
allocation and complete unfiltered suites remain required. Run all new initial
generation SQL/control/race work BEFORE the original terminal current-cohort
restart. That restart still proves complete canonical/shared/history equality,
actual unavailability and the same adopted cluster. Legacy schema 2 recovery
and both final owned cleanups remain unchanged.

The new row channels and scheduling guards do not enable product/bootstrap
writers, pools, HTTP authority, external dispatch or financial delivery. Actual
owning and public-only standalone source/native/compiler/model/process/profile/
counter/merge/conservation/retention/cleanup/strict/whole qualification remains.
All coverage floors, measured roles, complete denominators, finite capture/time/
storage bounds and original independent review/delivery gates remain mandatory.
