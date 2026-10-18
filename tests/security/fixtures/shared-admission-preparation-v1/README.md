# Shared admission input preparation boundaries

These original public synthetic literals use MIT OR Apache-2.0. They extend the
existing shared admission fixtures without changing their bytes or meanings.
The owning requirements remain resource-profiles, storage-and-search, API,
security-and-privacy, identity-contract, data-model and testing-and-coverage.

Only common_inputs and case.input may reach preparation. A complete case.input.common,
when present, replaces common_inputs; no partial merge is performed. Every case ID,
expected object, file pin, count, rights or profile field stays outside runtime
decisions. No input or expected object substitutes for actual canonical authority.

shared_preparation observes the existing original-claim input normalization,
finite control/outcome preparation and supplied Clock diagnostics. Normalization
preserves the entire supplied command and constructs only the selected caller
and three declared Clock samples. It is not typed claim admission, authentication
or authorization. Missing required command text or a missing/nontext selected
domain fails. Clock diagnostics observe one actual call to the supplied test
Clock, including its finite Display and Debug error representations; they do not
qualify wall-clock health. Invalid declared timestamps fail preparation.

ordered_claim_preparation applies the SAME complete input preparation used before
opening either session in the actual ordered-claim schedule. Exactly two commands
are required: claim_lock_validate_commit followed by wait_lock_recheck_deny or
wait_lock_recheck_admit. Both caller/claim identities, selected domains, clocks and
policy must be representable. Invalid input fails before a database session.
Both admitted second-operation names represent schedules only. Preparation does
not decide capacity, successful admission, locking, authority or a race result.

rls_preparation applies the SAME finite query, bound-selector and context
preparation used before the actual RLS probe opens its observation connection.
Its closed relations are organization_budgets, reservations and
reservation_recognition. Its five closed contexts intentionally omit one scope
dimension or select a distinct organization/principal. All fifteen supported
compositions remain representable; no SQL execution or expected denial is
inferred. Required selector text, selected-domain lookup and required distinct
context must exist. For foreign-principal selection, nontext or equal IDs are
skipped; absence of a distinct text ID fails. This does not validate UUID syntax,
database existence or authority. Actual PostgreSQL performs its separate typed
bind, existing-target and scoped-row checks.

The literals cover complete valid inputs and isolated missing, malformed, unknown
or structurally unsupported variants. Every prepared false is a preparation
failure, never a database denial, accepted operation, successful empty read or
cleanup proof. The consumer must use the same preparers in native helpers and
retain existing real SQL/lock/rollback/restart observations.

All original fixtures, case IDs, expected states, collection bounds and strict
measured coverage requirements remain unchanged. Complete unfiltered owning and
fresh public-only standalone qualification, current-image model reconciliation,
separate process probes, native/process/profile/source/resource conservation,
retained evidence, both owned cleanups, independent source/expected review,
distinct delivery, whole acceptance and real consuming gates still apply.
No product writer, actual authentication, human, live-model, recovery, user or
production requirement is qualified by these preparation literals.
