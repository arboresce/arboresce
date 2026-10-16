# Principal directory port qualification inputs

These original public synthetic literals are under MIT OR Apache-2.0. They
contain no personal records, operational credentials or verified signed events.
They extend the five unchanged `principal_mapping` cases in the independently
reviewed [authority package](../authority-v1/README.md). Ordinary checks read
committed literals; this package contains no maintained generator or decoder.

[cases.json](cases.json) pins complete owning public sources and the original
authority package. Inputs and expected observations are separate. The explicit
reading limits apply before JSON decoding; all listed original and additional
case identities are required. Expected values never enter the directory port,
query, record constructor or mapping function.

The bounded application boundary is selected as follows:

- `IdentityKey::parse` checks the exact selected profile and version, then exactly
  64 lower-case hexadecimal bytes and retains the resulting 32-byte public key.
  Its accessors return that exact profile, version and byte array. This is a
  checked lookup key, not admission of a complete wire identity profile or proof
  of curve validity, signature verification or key control. Zero and maximal
  byte strings deliberately qualify representation only.
- `StoredPrincipal::decode` admits the stored identity tuple and canonical
  UUIDv4 principal ID. Any malformed or unsupported stored value returns finite
  `DirectoryFailure::InvalidStoredData`, without echoing input or a raw cause.
  Twelve `stored_cases` observe this constructor directly, including every
  selected stored-value discriminator. They admit a valid other-key record and
  retain its exact key; the separate mapper must reject that record for a
  different query. A later mapping failure cannot stand in for decoder rejection.
- `PrincipalDirectory::lookup` receives the complete checked key and returns an
  asynchronous typed `Missing`, `One(StoredPrincipal)` or `Multiple` result, or
  finite `Unavailable`/`InvalidStoredData`. A backend must distinguish zero rows
  from failure and stop at two observations; two identical IDs are still
  ambiguous. It exposes no creation, alias-selection or assurance API.
- `map_principal` actually invokes that port once. It admits only one returned
  record whose complete identity equals the query, denies absence and ambiguity,
  and propagates backend failure. A Party alias cannot change the query or
  select a fallback. A pending port future must complete before a result exists.

The original trusted lookup observations feed an actual test implementation of
the port. They are not passed to a function choosing caller-supplied IDs. The
additional rows exercise the real checked stored-record constructor. The
observer records the actual query, call count, future polls and typed result;
assertion bodies retain the expected answers. Original no-creation/assurance/
cryptography fields remain scope invariants of this read-only mapping boundary,
never evidence of live registration or actual human approval.

The six `PrincipalFailure` and two `DirectoryFailure` variants have finite Rust
diagnostic labels. Each error case requires exact Display, no underlying source,
preserved typed downcast through `Box<dyn Error>` and propagation of a real
formatting-sink failure. These labels introduce no HTTP problem or response body.

This component selects no SQL global-lookup mode, RLS bypass, credential, clock
or I/O timeout implementation. The first actual authentication consumer still
requires a qualified real directory adapter, exact NIP revisions/libraries and
signed vectors, bounded service deadlines and real current-authority checks.
Mapping an ID grants no workspace/action/delegation/disclosure or human authority.
Current-policy/final-effect fences, real races, bootstrap, shared admission,
pooling, authentic lifecycle continuity, recovery and whole E205 remain required.
Product and bootstrap writers stay disabled.
