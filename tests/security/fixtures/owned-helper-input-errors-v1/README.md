# Owned helper input errors

These original public synthetic literals use MIT OR Apache-2.0. The existing
[resource](../../../../docs/requirements/resource-profiles.md),
[storage](../../../../docs/requirements/storage-and-search.md),
[security](../../../../docs/requirements/security-and-privacy.md) and
[testing](../../../../docs/requirements/testing-and-coverage.md) requirements own
their meaning. Three cases specify rejection of malformed close selectors.
One case observes an actual missing PID file inside a producer-owned directory.

The close_faults group keeps constructor failure distinct from a completed
guard that rejects its data. Each complete expected object represents the outer
Result after its actual error has been converted to text:
{"Err":"the exact error"}. An Ok value containing accepted:false cannot satisfy
an expected constructor error. A different setup, observation or cleanup error
cannot pass merely because it also fails; the entire error must match.

| Close input condition | Required outer error |
| --- | --- |
| The fault selector is absent. | missing close package fault |
| The fault selector is a number. | missing close package fault |
| The text selector is outside the admitted finite set. | unadmitted close package fault |

Missing or nontext selectors reject before an owned copy is created. The
unadmitted text selector uses the existing producer's exclusively created copy
of the fixed [transport package](../owned-transport-close-v1/README.md); its
original constructor error must survive owned cleanup. The source package is
unchanged.

The owned_pid_files group uses that same existing owned-copy and observation
boundary with the unchanged selector. A fixed caller invokes the actual PID-file
reader at postmaster.pid inside the owned directory. That package contains only
README.md, cases.json and profile.json, so no such PID file exists. The fixed
comparison PID is 17. Neither path nor comparison PID comes from the fixture;
neither identifies a live process. The caller never signals or adopts a process.

For the selected non-root macOS ARM64 Rust 1.99.0 profile, the complete expected
semantic result is {"accepted":false,"failure":"No such file or directory (os
error 2)"}. This is the actual reader's missing-file error propagated through
the existing guard-result boundary. The outer producer must succeed, preserve
the complete before/after package images and positively clean up its owned
directory. A constructor, image or cleanup failure is a test failure. This
finite filesystem case establishes no PostgreSQL error or server qualification.

Only case.input reaches its corresponding existing producer or fixed facade.
Identity, expected results, the outer profile, rights, pins and counts remain
outside its decision. The empty common_inputs object adds no runtime value.
After return, the caller compares the entire outer Err object for close_faults
or the entire actual semantic object for owned_pid_files. It retains the actual
observation and emits each event only after the assertions succeed.

No case selects a source, output or cleanup path, connection string, external
process or service. Existing closed selectors, producer-owned paths and bounds
remain. These cases add no Rust source file, dependency, feature, role exclusion,
target, test body or resource budget. Their four events extend the existing
thin helper harness. All original helper, package-guard, controlled-close and
actual PostgreSQL-close cases and complete expected objects remain unchanged.

Static admission qualifies these literal meanings and public provenance.
Every original fresh owning and public-only standalone collection, strict
coverage, source/compiler/native/process/profile/resource/conservation gate,
both cleanup, current-image model, separate default and 1 MiB native probes,
independent source and expected review, whole-checkpoint acceptance, verified
normal commit and actual distinct delivery requirement remains.
