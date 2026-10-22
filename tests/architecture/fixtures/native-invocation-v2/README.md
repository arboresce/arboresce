# Native invocation identity inputs

This package defines the next explicit native measurement protocol before its
consumer. It does not qualify execution, LLVM decoding, source coverage, resource
proofs or delivery. Supplied facts and PIDs are representation inputs. Real runs
must retain actual owned `Child` identities, statuses, files and complete inputs.
The earlier packages and their PID-unique rejection meanings remain unchanged.

## Selection and immutable context

`collect-v2` explicitly selects `native-measurement-v2`, `native-launch-v2`,
`native-exit-v2`, `native-invocation-v2` and `native-collection-facts-v2` together.
The existing unprefixed five-argument collection form, `postgres`, `prepare`,
their help/usage and v1 readers retain their existing meanings. The new exact
routes are `collect-v2 <workspace> <roles.json> <tests.json> <prepared.json>
<fresh-output>` and `collect-v2 postgres <workspace> <roles.json> <tests.json>
<prepared.json> <postgres-input.json> <fresh-output>`. `collect-v2 --help`
prints only the two new usage lines and exits zero. A malformed new route prints
those lines to stderr and exits two. Non-UTF-8 intake, output failures and
collection diagnostics retain the existing distinct exit-one meanings.
Mixed or ambiguous current protocols fail. No historical receipt is rewritten.

The closed runtime context has exactly `version`, `run_id`, `source_sha256`,
`driver_pid` and `profile_directory`. Version is `native-measurement-v2`; the
run ID is lowercase 32-hex, source digest lowercase 64-hex, driver PID a positive
u32 and profile directory a canonical absolute UTF-8 byte-array path without
NUL or literal percent bytes. The directory must already exist and be regular
directory intake, not a symlink. Rejecting percent prevents directory bytes
from becoming LLVM substitutions. Its raw bytes,
path and SHA-256 remain immutable. Existing directory, regular-file, intake,
source, compiler, native, resource, process and coverage limits still apply.
The context owns separate flat `invocations`, `launches` and `exits` directories;
their per-invocation sets must agree completely in a green run. Unused allocated
slots, missing records, foreign files and changed bytes cannot be omitted.
Each flat directory retains the existing inclusive 4,096-file/65,536-byte
per-file/256-MiB aggregate bound. All three directories and `context.json`
share the original enclosing inclusive 8,193-file/65,536-byte per-file/512-MiB
aggregate/depth-one bound. Thus a complete v2 run has at most 2,730 invocations;
the next complete triple exceeds the enclosing file budget and fails intake.
Adding reservations does not multiply a limit. The context file retains its
separate 32,768-byte input bound.

## Invocation allocation and actual child ownership

An invocation ID is an opaque lowercase 64-hex run-scoped identity. It is
allocated once before spawning, exclusively reserved and never recycled or
overwritten. Actual OS PID is separate metadata and may recur across lifetimes.
The parent invocation is null only for the declared actual driver. Every other
parent invocation identifies the exact recorded owning process, with matching
actual parent PID. Parent chains must be acyclic and reach the driver. An
immediate child cannot have its live parent's PID; a later descendant may reuse
an exited ancestor's PID without becoming that ancestor. No descendant may have
the still-running driver PID. A reservation has no child PID: child PID agreement
applies to launch, exit, command capture and profile, while the reservation
binds its actual parent PID.

The selected allocator uses a checked positive process-local u64 sequence and
the parent's invocation identity, rather than PID or wall-clock uniqueness.
Its exact token preimage is the concatenation of:

1. UTF-8 `arboresce.native-invocation.v2` followed by one zero byte;
2. the decoded 32 bytes of the immutable context SHA-256;
3. one byte, zero for the driver parent or one for an invocation parent;
4. 32 zero bytes for the driver parent, otherwise the decoded parent ID;
5. the positive sequence as eight big-endian bytes.

The token is lowercase SHA-256 of the complete preimage. The sequence advances
once per preparation, including across threads. The complete `(parent,sequence)`
pair is unique within the run. Overflow, a duplicate reservation or failed
reservation write fails before spawning; no collision retry or discarded
invocation can manufacture a green run. Exclusive reservation remains the
nonreuse enforcement even if the allocator cannot produce a fresh token.
This is measurement identity, not a signing, authentication or secret-key scheme.

Each reservation wrapper retains its actual raw SHA-256 separately from its
record. The record has `version: native-invocation-v2`, the four common identity
fields (`run_id`, `source_sha256`, `context_sha256`, `context_path`),
`invocation_id`, `sequence`, `parent_invocation_id` and actual `parent_pid`.
Physical reservation and receipt filenames are exactly `<invocation_id>.json`.

The actual owned Child carries its invocation, selected immutable request and
allocation/launch digests through observation, completion, failure and cleanup.
Completion cannot rediscover a lifetime by PID or select the latest receipt.
It derives status from that same Child. Repeated completion is byte-identical
and idempotent only for that bound invocation; conflicting records fail. An old
handle cannot complete a later lifetime, even with the same PID and command.
Existing raw-Child v1 APIs remain v1; an unbound raw Child cannot attest v2.
If an ambient or explicit selected request is v2, a raw-Child spawn API rejects
it before spawn. An explicit v1 request still takes precedence over ambient v2.
Bound-child APIs retain the selected request independently of later ambient
environment changes and use that same binding for successful or failed cleanup.

## Launch, exit, captures and raw profiles

A v2 launch retains all existing actual executable/path/SHA, program, argument
and cwd bytes. It additionally binds `invocation_id`, `sequence`,
`parent_invocation_id`, `allocation_sha256` and `raw_profile_pattern` alongside
the actual PID, parent PID and four common context fields. Its allocation fields
and digest must match the exact reservation. Before execution, child environment
selection carries the exact context and its own invocation identity. The actual
command capture includes those final environment values; preparation and spawn
cannot use different command bytes. Explicit nested requests take precedence
and cannot inherit another context's parent identity or profile namespace.

Normal bound preparation uses `observation_mode: selected`. It rejects an
explicit removed or different child `ARBORESCE_NATIVE_CONTEXT` before allocation
or spawn; otherwise it sets the selected context and fresh profile carrier.
The launch additionally records `child_context` as its final effective context
bytes. This preserves a distinction between immutable observation ownership
and the child's own later activation.

The separately selected native-fixture `as-configured` capture adapter exists
for the earlier deliberate removed, invalid and local-v1 child-context challenges.
It preserves those exact effective child context bytes (or null for absence),
retains `observation_mode: as-configured` and `child_context` in the v2 launch
and saved command intent, and always sets the fresh selected owner's LLVM
carrier. The actual owned Child, reservation, launch, exit, status and profile
still bind that outer owner. This mode supplies observation only: it cannot
qualify selected-context activation, replace a v1-local descendant receipt or
change an earlier expected result/error. The final complete effective context
and carrier values are captured before execution, with no parent-env mutation.
A captured as-configured peer cannot use its fresh carrier alone to activate
normal v2 descendants. Self-activation requires its own launch's selected mode,
actual matching effective selected-context bytes and actual PID, or a separately
declared new context for which this actual process is the driver. Supplied facts
also reject an as-configured invocation used as a v2 parent. No deadline, native
case, receipt, profile, raw-set or source-scope requirement is removed.

`ARBORESCE_NATIVE_CONTEXT` remains the canonical selected context directory.
The existing `LLVM_PROFILE_FILE` is the self-invocation carrier: its entire
UTF-8 value must be the exact selected-context pattern below, with the child's
own lowercase 64-hex ID. There is no new ambient key and no latest-PID lookup.
When the current process PID equals the selected context's declared driver PID,
parent is null and an outer context's pattern is ignored for this new context.
Otherwise preparation extracts the current process's ID only from its actual
ambient selected-context pattern, validates its exact reservation and immutable
launch against that context, selected observation mode, actual effective context
and current PID, and selects that ID as the parent.
The launch can be published just after the child begins: activation may poll
only a missing own launch, at one-millisecond intervals, for at most one second
and never beyond the original caller's remaining deadline. Changed, malformed
or conflicting records fail immediately. No spawn, allocation or command is
retried. Expiration is an actual activation failure within the original budget.
The launch's own raw-profile pattern must equal the actual ambient carrier.

An explicit B request may be a new context driven by the current process or a
validated B descendant. An A carrier cannot supply B's parent. Preparation sets
both context and fresh child pattern on that command before its intent is
captured, without mutating the parent environment. All four explicit outer/current
combinations (v1/v1, v1/v2, v2/v1, v2/v2) are permitted; records within each
context remain uniformly its selected version. Existing prepared-tool manifest
fields and the existing instrumentation-key allowlist remain unchanged. Outer
admission selects the declared context's explicit version, validates the actual
collector's own launch by the corresponding PID or invocation identity, and
retains exact executable/program/argv/cwd/environment checks. It never attests
its own final exit or profile. Existing v1 fixture meanings remain intact.

The exact raw pattern is `<profile_directory>/arboresce-<invocation_id>-%p-%12m.profraw`.
Each observed filename is `arboresce-<invocation_id>-<actual_PID>-<module>.profraw`,
where the nonempty module signature retains the existing ASCII alphanumeric/
underscore grammar. Separate lifetimes of the same PID and instrumented image
therefore have separate files. No profile is merged twice, overwritten, inferred
from another lifetime or credited without actual LLVM decoding. Raw module and
object identity, positive/zero activity rules, complete raw-set conservation,
mtime/size/digest stability and all native/owned-source scopes remain mandatory.

A v2 exit binds the same invocation, sequence, PID, parent invocation/PID and
common context fields plus the exact raw launch digest. Exit is one actual Unix
exit code 0..255 or positive signal, as before. Allocation and launch recording
failures retain their exact phase/cause and actual owned cleanup. No status is
invented, receipt replaced, child retried or native deadline increased.

## Complete fact conservation

Facts v2 retain all existing object/input/command/profile/native-scope categories
and add `invocations` wrappers. The context identity also declares
`measurement_version: native-measurement-v2` and `profile_directory`. All three
wrapper maps are keyed by invocation ID; actual PID and parent metadata must
agree across the linked records. Supplied wrapper hashes and opaque IDs do not
attest physical intake or the allocator's execution. Separate token literals
qualify the exact producer mathematics; real collection qualifies their origin.
Physical v2 reservation intake additionally recomputes the ID from its immutable
context digest, parent ID and checked sequence and compares the exact filename.
The pure fact validator intentionally checks opaque supplied identity links,
not physical allocator origin: this permits cycle and relabeling adversaries
without pretending to have generated a SHA-256 fixed point. Such unit facts
cannot qualify real collection. Duplicate `(parent,sequence)` pairs fail both
layers. Reservation, launch and exit wrapper records are closed to their defined
fields; unknown record fields fail rather than creating an alternate protocol.
The exact reservation fields are the four common identity fields plus `version`,
`invocation_id`, `sequence`, `parent_invocation_id` and `parent_pid`. The exact
launch fields are the four common identity fields plus `version`, `pid`,
`parent_pid`, `executable`, `executable_sha256`, `program`, `arguments`, `cwd`,
`invocation_id`, `sequence`, `parent_invocation_id`, `allocation_sha256` and
`raw_profile_pattern`, plus `observation_mode` and `child_context` as defined
above. Selected mode requires context-path equality; as-configured preserves
its supplied effective bytes or null without asserting activation. Any other
mode, missing field or non-byte context representation fails. The exact exit
fields are the four common identity fields
plus `version`, `pid`, `parent_pid`, `invocation_id`, `sequence`,
`parent_invocation_id`, `launch_sha256`, `exit` and `signal`.

The token API takes exactly `context_sha256`, `parent_invocation_id` and
`sequence`. It rejects unknown keys, invalid lowercase digest spellings and
non-integer/zero/out-of-u64 sequences, including JSON booleans and `1.0`.
Token literals freeze both the complete preimage bytes and resulting digest.
Fact case patches are JSON-pointer assignments to the named baseline scenario;
an existing parent object may receive a new final member, but an absent parent
or nonexistent array index is a preparation error. No expected value enters
consumer selection. The baseline copies the existing public v1 object/input
facts and adds separate same-PID binary/tool lifetimes. Nested facts separately
exercise an exited ancestor PID without conflating invocation identity.

Commands and profiles add `invocation_id` and keep actual PID. Exact harness
list/run roles, arguments, object and parent identities, complete required suites,
declared descendant probes and case-event attribution still apply. Event lines
retain actual PIDs; their exact invocation is bound by the owned capture.
Native binary scopes select `profile_invocations`, the complete distinct set of
their exact object's invocation IDs, rather than `profile_pids`. A legacy scope
or PID-only fallback cannot qualify v2. Every prior source/count/strict threshold
and full before/after category remains required; expected errors retain the
existing input, changed-input and conservation distinctions.

Deterministic duplicate-PID facts are unit inputs, never claims of forced native
PID reuse. Actual sequential/nested children, receipt conflicts and failures,
profile substitution and complete standalone collection still need qualification.
This package neither enables a product writer nor supplies lifecycle continuity.

Runtime recipes select already prepared native tools, real owned children and
task-owned contexts. Their PID/status placeholders are materialized only from
those actual observations. The missing-executable recipe fails before execution;
the conflicting-launch recipe publishes its exact conflicting token path after
reservation and before spawning the selected thirty-second sleep, so only that
actual owned child is killed/reaped. Reservation-conflict preparation uses a
known token under a fresh process-local sequence and creates that exact path
before preparation; it never retries allocation. Delayed publication deliberately
holds only the parent's actual launch publication for twenty milliseconds.
The unchanged two-second kill/reap bound applies. The byte-limit identity cases
check supplied size-map arithmetic; physical walker limits, exact raw bytes and
actual collection remain separate mandatory checks. An actual successful run
cannot credit an orphan reservation, failed activation or replaced raw profile.
