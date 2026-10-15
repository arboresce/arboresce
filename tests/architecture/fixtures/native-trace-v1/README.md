# Complete native trace inputs

These original public synthetic byte recipes and literal expected results use
MIT OR Apache-2.0 terms. They independently preserve the existing complete
observation trace format, captured8MiB limit on each stream, all existing
300-second required-harness and180-second collector phase ceilings, owned-child
cleanup and physical completion requirements. They introduce no new runtime cap.

A real controlled test peer emits both prepared files. Exact byte arrays include
NUL and non-UTF8 values; they stay numeric arrays in trace JSON. Fixed trace
lengths/SHA256 replace only the actual PID in its three declared positions with0.
The full two-stream boundary expands to roughly64MiB of trace; it must retain all
bytes. Loose deterministic writer-call ceilings prove bounded transport work
without requiring a machine-specific timing threshold or exact chunk split.
Short writes preserve every byte. Each stream's one-over case rejects independently.

Faults test a first write, a partial frame, explicit flush and a successful flush
that returns after the operation deadline. Discard pending observation bytes on
failure; hidden buffer-drop retries must not precede required cleanup records.
A first successful write after a one-shot IO error starts explicit cleanup.
Serialization failure must not trigger a second flush or replace its IO kind.
The flush-failure writer rejects a second flush differently to discriminate that
error boundary. A writer-work exhaustion remains failure and still reaps the
actual child and records its completion. Captured streams remain physically exact.

Measured support retains the task-owned input, captured streams, trace and
explicit receipt files for independent review. The explicit context has synthetic
selectors but records a real process, executable hash and observed status;
its completion is absent before observe and mandatory afterward. Ambient source
context, when active, retains its own conservation independently. No expected
answer is sent to the peer or observer. Existing late/error/cleanup tests remain.
Independent expected/context acceptance precedes repair and consumption; exact
source plus fresh owning/standalone/native qualification still follows.

Three additional preparation cases exercise actual post-spawn failure handling.
An actual create_new probe must prove owned launch-directory write denial before
spawning the already-selected /bin/sleep10. The child must still be running
before failed B recording and must actually die by signal9 and be reaped; a
matching B pair cannot be fabricated. Restore the original owned permissions.
The other real test-peer children exit0 before an owned empty exits-directory
rename or occupied trace-directory causes the original NotFound/AlreadyExists
preparation error. The renamed directory remains retained as evidence.

Each failure independently attempts owned termination, B completion, a separate
real fallback-context completion and ambient completion where active. The
fallback physical launch/exit binds the actual executable, PID, launch digest
and observed status even when B completion fails. An observe-entry marker is
absent; no tested observer runs. Assertions receive the actual preparation Err
and retained diagnostic path, then inspect the real receipts/streams. The
existing2-second cleanup bound applies, including recording's own cleanup.
These cases neither alter the previous14 literal answers nor permit incomplete
successful observation. They qualify tested owned cleanup, not product recovery.
