# Explicit native process measurement inputs

These literals define the first native launch-receipt boundary. Independent
expected-value review must precede consumer implementation. They do not qualify
the unfinished complete verification collector or any product workflow.

Measurement is requested only by `ARBORESCE_NATIVE_CONTEXT` in the parent process.
Absent activation preserves ordinary operation and writes no measurement files.
The selected directory is explicit and caller-owned. Its `context.json` is a
regular non-symlink file, at most 32768 bytes, with exactly version, run_id and
source_sha256. The version is native-measurement-v1. The identifiers contain
exactly 32 and 64 lowercase hexadecimal characters. Duplicate JSON keys, trailing
JSON and unknown fields fail. `launches` and `exits` must already exist as real
directories. A symlink at any of these four selected entries fails. The canonical
root binds the request; canonical ancestors may include operating-system aliases.

Before spawning, resolve the actual current directory, selected executable and
exact raw positional argument bytes. Executables must be regular executable files with at
most 67108864 bytes. The explicit selection function uses command PATH before parent PATH, including
its removal. Bare requests are bound to a concrete absolute selected path before
constructing the Command that will be prepared and spawned. Active preparation
rejects an unbound bare program. Selection retains a tool symlink spelling so
its normal basename dispatch remains valid. Kernel PATH fallback cannot replace
the selected image; a selected but inaccessible file fails closed at actual spawn. A relative program uses the actual command working directory. Bind the
canonical executable bytes to SHA-256. The actual bound Command retains its configured PATH for its own descendants.
Missing PATH does not enable a fallback.

A successful spawn writes an exclusive `launches/<pid>.json` record containing
version native-launch-v1, canonical context path and context digest, run/source
identities, actual child and parent PIDs, canonical executable path and digest,
original program spelling, exact raw positional arguments (excluding argv[0])
and canonical working directory. Path, program and positional argument values are byte arrays, preserving
non-UTF-8 Unix values. Custom argv[0] overrides are outside this initial profile;
the maintained call sites do not use them. Preparation and spawn must use the
same unchanged Command. The program field names the actual bound Command spelling; selection of a bare
request precedes its construction. Both use the same inspected PATH/cwd context.
Canonical executable identity and original program spelling are retained
separately; canonicalization does not rewrite the child program spelling. A post-spawn receipt failure kills and
reaps only that owned child within two seconds before returning failure. Prior
receipts and evidence are preserved.

Actual completion writes exclusive `exits/<pid>.json` containing version
native-exit-v1, matching context/run/source/parent/child identities, real exit code
and signal, obtained directly from that same actual Child. An unreaped/running
child cannot supply a completion and returns WouldBlock. The matching launch must still exist and belong to this request and
parent. Repeating the same completion is idempotent and preserves bytes; a
conflicting or symlinked completion fails. This supports observing the same
already-reaped Child twice without inventing another process/profile.

The shared bounded process observer records actual completion even when stream
or trace collection fails and after owned-child cleanup. Delegated status and
captured output use bounded observation, preserve actual streams and exit status,
and fail closed if recording fails. The test-only peer exercises these paths
using real operating-system executables. These controlled executions do not
qualify Cargo, service, runtime or complete collection behavior. Actual Rust
process/profile reconciliation remains mandatory at the collector gate.

The oversized context recipe pads otherwise-valid full context JSON with ASCII
spaces to exactly 32769 bytes; the 32768-byte positive discriminates the boundary.
The executable size cases use regular executable files at exactly 67108864 and
67108865 bytes. The exact-limit case qualifies only preparation/hash intake;
it does not claim that an arbitrary padded file is a valid native executable.
The launch-identity mutation changes only the run_id to the frozen different
32-character value while preserving valid JSON and all other launch fields.
The launch collision uses one actually running /bin/sleep child. It preserves
the pre-existing collision bytes, asserts running state before recording,
observes actual signal 9 and bounded reap, and counts one invocation.

Each case is executed and reported once by its literal ID. Expected error kinds,
exit codes, streams, identity equality, immutable receipts and owned cleanup are
asserted directly. File/executable builders are measured source. No expected
value is generated from the consumer.

The active missing-output and broken-trace native cases retain one matched
launch/exit receipt pair for the actual external child exiting 0, while the
parent reports the fixed collection failure and exits 1. Missing output removes
only its owned stdout capture file. Broken trace uses a task-owned closed pipe
with its reader dropped, so trace writes fail without mutating unrelated state.

The selected-path discriminator requires the current non-root Unix user. Its
first PATH directory contains a copy of the prepared true executable with mode
0401 (owner read, no owner execute); a later directory contains a valid executable
copy with mode0755. Selection may choose the first candidate, but the actual
Command uses its absolute path, so PermissionDenied must fail before a Child or
receipt exists. It cannot run the later image while recording the first hash.
Temporary copies remain in task-owned external fixture storage and are deleted
with that storage; no operating-system payload enters the repository.

The cached-late status case first observes the same true Child exiting0 using
bounded preparation, then rejects its zero-budget tested observation as TimedOut.
It retains that actual0 status, same PID and one invocation. The status deadline
is checked before accepting an outcome and after completion recording.

Normal delegated Cargo operations use explicit action budgets: format60 seconds,
check180 seconds, test300 seconds and Clippy180 seconds. These allow the already
observed complete test/build work rather than imposing a native-fixture30-second
limit. Metadata/captured native fixture observation remains30 seconds, and owned
cleanup remains2 seconds. Each future larger workspace must qualify its actual
commands and operation budget; no timeout can silently become a passing run.

The builder-error cases exercise maintained preparation support directly, without
claiming that a preparation failure proves consumer rejection. The selected PATH
positive also preserves raw argument bytes and an explicit unrelated environment
removal when constructing the bound Command. The selected missing-tool native
case requires factory failure before a Child or local receipt exists.

The context-change case prepares one Command against the original valid request,
then spawns one real running sleep Child. Before recording, it changes only the
context run_id to the frozen different valid value. Recording must fail
InvalidData, kill/reap that same owned Child with signal9 within2 seconds, and
leave zero local launch/exit receipts. The changed context remains intact; no
receipt is falsely rebound to its new identity. Outer qualification records and
profiles remain separate and are never removed or replaced by this fixture.
