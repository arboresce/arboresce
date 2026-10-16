# Explicit migration service and relay inputs

These independently reviewed original synthetic literals precede the native
service-input helper and protocol relay. They extend the existing migration
scenarios without changing any accepted record, scenario, constraint or budget.
All material is under MIT OR Apache-2.0.

The service structure names one exact prepared local PostgreSQL input: protected
Unix socket and owned data directory, initial process/start/system identity,
PostgreSQL 17.11, native ARM64/OS build, finite lease, separately bound accepted
readiness evidence, two exact native tools and all selected configuration/native
file identities. Required configuration paths are relative to the declared data
directory. Both tools must occur with the same hash in the file map. Only these
fields and tool names are accepted. Paths are absolute, without parent traversal,
NUL or symlinked selected files/ancestors. The socket path is at most 80 bytes.
Data and socket directories are actually mode 0700. No ambient selector can
replace the explicit collector argument. Before commands run, remove the ambient
service selector and every PG-prefixed environment key, then inject only the
selected input path. The adapter additionally rejects ambient PG variables before
constructing SQLx options without password-file access.

The service JSON is at most 2 MiB. The separately selected native-input map has
at most 64 regular files, each at most 2 GiB and at most 8 GiB in total. These
limits cover the actual macOS dyld cache and subcache files; they do not change
the existing compiler-resource, raw-profile or source-discovery limits. Reopen
and hash the complete service/profile/configuration/native/readiness inputs
before and after qualification. An expired lease or changed/missing identity
fails. Structural positive fixtures contain synthetic hashes and prove shape
only; they never assert an actual server or actual file identity. The real
service checks independently verify the selected server and non-owner role facts.

The relay has exactly one owned local client/server pair and at most two
forwarding threads. It forwards real traffic. The server direction reads the
one-byte type, four-byte big-endian length (including those four bytes), then the
declared body. The complete frame, including its type, is at most 1 MiB. Invalid
lengths fail before allocation. CommandComplete bodies must be nonempty,
NUL-terminated and contain no earlier NUL. Only an exact COMMIT or ROLLBACK tag
in its selected loss mode is withheld; unrelated or opposite-mode frames pass
unchanged. Closing both owned streams after withholding leaves the adapter
uncertain. Independent direct reads establish the actual committed/rolled-back
state. No fake transaction response qualifies these service scenarios.

Retain the 8 MiB limit in each direction, 4096 server-frame limit, 2-second I/O
limit and 10-second total operation deadline. Each I/O timeout is no greater than
the remaining total budget. Required input absence is a failure, never a skip.
Every maintained reader, relay, lifecycle and observation helper is measured;
assertion-only bodies remain excluded. All actual native command launch/exit
receipts and indirect PostgreSQL generation observations remain required.
An owned restart preserves system identity/configuration and the useful typed
row while establishing a distinct actual process/start generation. It is a
persistence/reconnect scenario, not packaged recovery or production admission.

The physical input recipes use exclusive task-owned regular files and protected
directories. Their bytes and paths are synthetic; their filesystem operations
are real. `change-profile` compares a complete before/after observation after
adding JSON whitespace; the changed input hash must prevent unchanged acceptance.
The environment recipe launches the maintained peer after explicit selection,
with an ambient selector and PG defaults supplied only to that owned child.
These helper cases cannot qualify server metadata, transactions or recovery.

## Owned restart handoff

The prepared service's controller remains outside the build lease. The test runs
one restart last in each fresh service, under a 25-second handoff that fits the
original lease. Stop, disappearance checks and their acknowledgement together
fit the existing ten-second shutdown limit. Initial input bytes and generation
remain immutable; only observed successor PID/start state advances.

Control files live in the protected parent of the declared data directory. Each
is an atomically published, create-new regular JSON file of at most 4096 bytes.
Reads must reject symlinks and nonregular inputs without blocking. The request
`restart-request-v1.json` contains version
`postgres-harness-restart-request-v1`, the actual `input_sha256`, `old_pid`,
`old_start` and `deadline_unix_ms`. The controller uses its fixed owned namespace
and tools; marker strings cannot select another process or directory. It
acknowledges the exact request SHA256, input hash, old generation and deadline in
`restart-ack-v1.json` before stop begins.

After actual old-process, PID-file and socket disappearance, the controller
publishes `restart-stopped-v1.json` with the request hash, old PID and the three
affirmative absence observations. Only then may the measured test start the same
cluster. The controller reserves verified successor cleanup ownership as soon as
it becomes visible, including after a failed or timed-out native start. The test
accepts a new generation only after same-cluster metadata, current role authority,
a fresh exact typed read and the unchanged migration/row/annotation snapshot.
Its `restart-complete-v1.json` binds version, request hash, old/new PID, new
postmaster start and unchanged system identifier. The controller's
`restart-adopted-v1.json` binds that complete file's exact hash and successor
identity. Their versions are `postgres-harness-restart-stopped-v1`,
`postgres-harness-restart-complete-v1` and
`postgres-harness-restart-adopted-v1`; acknowledgement uses
`postgres-harness-restart-ack-v1`.

Retain failed commands, partial streams, observed native Child outcomes and
bounded cleanup records. A daemon's indirect observations never become a
fabricated Child exit status. Failure to verify current ownership prohibits
signaling a cached PID. A completed restart consumes that initial service
generation; another qualifying run requires a fresh prepared service and input.
