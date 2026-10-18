# Owned transport fixture package guards

These original public synthetic literals use MIT OR Apache-2.0. Their owners are
the existing [resource](../../../../docs/requirements/resource-profiles.md),
[storage](../../../../docs/requirements/storage-and-search.md),
[security](../../../../docs/requirements/security-and-privacy.md) and
[testing](../../../../docs/requirements/testing-and-coverage.md) requirements.
They exercise the integrity checks for the accepted
[owned transport close package](../owned-transport-close-v1/README.md).
Every original fixture and complete expected object remains unchanged.

The unchanged control and fifteen corruptions use an exclusively created,
independent three-file copy of that fixed package. The producer owns its paths,
bounded preparation, mutation, complete before/after observations and cleanup.
A case supplies only its finite fault selector. It supplies no path, PID,
connection string, external target or cleanup target. All original source
package bytes remain unchanged.

Only case.input reaches the guard producer. The empty common_inputs object adds
no runtime value. Case identity, expected results, the outer profile, rights,
pins and counts stay outside guard decisions. The copied package profile is the
data under test. Its fields are validated as package input; the outer execution
profile and oracle are never used to choose a rejection. Each complete expected
object is compared after the producer returns.

Preparation, mutation, observation and cleanup failures remain test failures.
They cannot satisfy an expected loader rejection. The unchanged control requires
actual successful package intake. A rejected case requires the actual complete
loader failure recorded in cases.json, with the actual error retained. Copied bytes and
entry kinds before and after mutation make each finite change inspectable.

| Fault selector | Sole intended corruption |
| --- | --- |
| unchanged | Copy all three files without changing any byte. |
| nonregular_entry | Add an owned directory beside the three regular files. |
| additional_file | Add one owned regular file. |
| missing_readme | Remove only the copied README.md. |
| source_hash | Replace the first source SHA-256 pin with 64 zeroes. |
| source_length | Increase only the first source byte-length pin by one. |
| missing_source_path | Remove only the first source path field. |
| missing_source_pins | Remove the source-pin vector. |
| unregistered_file_pin | Add an unregistered file-pin name. |
| file_hash | Change one ASCII byte of the copied README.md at the same length. |
| file_length | Increase only the README.md byte-length pin by one. |
| missing_file_pins | Remove the file-pin map. |
| profile_version | Replace only the copied profile version with unsupported. |
| cases_version | Replace the copied case version with unsupported and bind that copied file's actual new hash and length. |
| missing_projection | Remove only the copied allowed-projection field. |
| unsupported_projection | Replace the copied allowed projection with case.input alone. |

The case-version mutation updates its copied file pin so the version check is
reached with truthful byte identity. Every other pin remains unchanged. All
other cases retain the original copied case bytes and expected objects.
Source pins continue to select the fixed public source; no mutation chooses a
different source path or changes a source file.

These finite filesystem guard observations preserve the sixteen-case and
existing file, JSON, source-read and observation bounds. They add no Rust source file,
dependency, feature, role exclusion, target, test body or resource budget.
Their sixteen registered case events extend the existing thin helper harness.
The five controlled task and six actual PostgreSQL close cases remain required.

Static admission qualifies these finite meanings and public provenance. Every
original fresh owning and public-only standalone, full unfiltered collection,
strict coverage, source/compiler/native/process/profile/resource/conservation,
both cleanup, current-image model, separate default and 1 MiB native probes,
independent source/expected review, whole-checkpoint, verified normal commit and
actual distinct delivery requirement remains.
