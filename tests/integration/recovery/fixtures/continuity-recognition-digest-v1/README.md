# Exact recognition digest literals

These original public synthetic inputs use MIT OR Apache-2.0. Independent literal
acceptance and committed delivery precede the pure application consumer.
[sources.json](sources.json) freezes source pins, limits, complete identities and
the prospective dependency/native inputs; [cases.json](cases.json) holds the
independent expected values.

The [lifecycle owner](../../../../../docs/requirements/lifecycle-contract.md#11-retention-recognition-and-independent-continuity)
and [common key/name syntax](../../../../../contracts/public/v1/common.schema.json)
govern the mathematics. A key has exactly 32..128 ASCII characters from
A-Z, a-z, 0-9, underscore and hyphen. Hash its exact ASCII bytes with SHA-256;
never trim, normalize, case-fold, decode base64, append a newline or add a domain.

A recognition identity has organization_id, principal_id, command and key_digest.
The two IDs and digest are already checked domain values. The command constructor
checks only common.command_name syntax: 3..64 ASCII characters, at least two
dot-separated nonempty components, each starting with a lower-case letter and
continuing with lower-case letters, digits or underscore. Format checking and
hash computation do not prove that a command is registered or admitted. The
wellformed-unregistered and zero/ff digest units explicitly have mathematical
scope only. Actual first consumers still require the registered admitted command
and its real key relation, current authorization and every canonical admission fence.

The digest is SHA256(UTF8('arboresce.command-recognition.v1') || one actual NUL
byte || JCS(recognition_identity)). For these four checked ASCII-only strings,
the complete fixed JCS object has keys in exactly this order: command,
key_digest, organization_id, principal_id. Their values need no escaping.
A fixed serializer for this bounded type does not establish generic JCS support.
Independent canonical UTF-8 and full preimage hexadecimal literals are review
evidence, never inputs to the runtime or helper. The separator is 00, never the
two printed backslash-zero characters. Domain, purpose, credentials and signing
keys are not part of this recognition identity or organization/principal scope.

The resulting reference contains only scope and recognition_sha256. Raw keys,
command preimages and canonical identity bytes remain transient; no key or
preimage is retained or logged by the library. The fixed generic error Display
and constant reference Debug expose none of those inputs. This finite diagnostic
observation is not a general privacy proof or a secure-memory erasure claim.

Only case.input reaches a maintained measured helper and consumer. Expected
objects, independent_encoding, case IDs and provenance remain in thin assertions.
Observe the actual result/reference/error/Debug and unchanged supplied input after
the call. No expected digest may select an implementation result.

The separate closed application dependency profile admits the already prepared
sha2 0.10.9 with force-soft; it selects no new archive or external feature.
The prospective source profile adds recognition.rs and its input-only support:
99 Rust sources, 88 measured, ten assertion sources, one declaration, 98 LLVM
rows and the same ten targets. The active 97-source profile stays unchanged until
a separate consumer. The separately frozen directed lock context is mandatory;
no general source-pin or snapshot exception is granted.

Static data qualifies no executable, authenticated journal, independently durable
acknowledgement, binding/abort/inventory interpretation, cutoff, recognition or
retirement admission barrier, recovery, operational trust or writer. Complete
fresh owning/public-only Rust 1.99.0 source/helper/process/native qualification,
strict required coverage, original-owner MODEL/probe gates, separate independent
reviews and actual distinct delivery with genuine owner ROOT receipts remain.
All whole-continuity and actual human/live-model/recovery/native/user/production
duties remain; product/bootstrap writers are disabled.
