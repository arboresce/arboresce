# Common contract checkpoint evidence

Date: 2026-09-08. Scope: E002's shared contract and independent parsed-value
fixtures. This document records contract verification, with no runtime pass.

The starting public revision was
`ae51d73d11efa658115d9b377fd489c9cfd4a129`. The change introduces a versioned
common schema, its owning wire contract, independent expected values and a
locked standalone upstream validator project. Existing rights notices and the
independent repository history are preserved.

The common schema is a definition library. Consumers select an explicit named
definition; shared metadata cannot authorize a complete domain command. Operation
body contracts and current-policy, reference, transactional and transport checks
remain with their owning implementation slices.

## Prepared validation profile

The actual prepared profile uses native macOS ARM64, CPython 3.14.7 and uv
0.12.10. The virtual project installs no first-party Python package. Its exact
locked upstream packages are:

| Package | Version |
| --- | --- |
| attrs | 26.1.0 |
| certifi | 2026.7.22 |
| charset-normalizer | 3.5.1 |
| check-jsonschema | 0.38.0 |
| click | 8.5.0 |
| idna | 3.19 |
| jsonschema | 4.26.0 |
| jsonschema-specifications | 2025.9.1 |
| referencing | 0.37.0 |
| regress | 2026.9.1 |
| requests | 2.34.2 |
| rpds-py | 2026.6.3 |
| ruamel.yaml | 0.19.1 |
| urllib3 | 2.7.0 |

Preparation used an isolated external environment and the applicable workstation
build-output routing. Host defaults remain unchanged. The supported portable
commands are in [the validator guide](../../contracts/validation/README.md).
Exact environment synchronization passed for all 14 installed packages, without
ambient optional parsers. Runtime processing dependencies are a separate profile.

An isolated validator probe checked the chosen mechanism before applying it to
the product contracts: local Draft 2020-12 definitions and an expectation schema
accepted one positive and one negative expected value, both with exit zero.
Inverting the expected rejection produced a structured validation failure;
removing the local referenced schema produced a reference-resolution failure.
Both failures exit nonzero, confirming why a negative fixture cannot simply
invert the tool's exit status. This probe is tool-selection evidence only.

## Contract and fixture verification

The four commands in the validator guide passed against the final inputs:

| Check | Actual result |
| --- | --- |
| `uv lock --project contracts/validation --check --offline` | Exit 0; 15 lock records, including the virtual project. |
| `uv sync --project contracts/validation --check --locked --offline` | Exit 0; 14 installed packages checked, no changes required. |
| `check-jsonschema --check-metaschema` with both explicit schema paths and the documented options | Exit 0; both Draft 2020-12 schemas accepted. |
| `check-jsonschema --schemafile contracts/validation/common-v1.expectations.schema.json` with the explicit case path and documented options | Exit 0; all 248 expected outcomes accepted. |

The finite inventory has 66 accepted and 182 rejected values across 17 named
targets. Its expected outcomes were authored separately from the common schema.
Cases include identifier/revision boundaries, closed command metadata, all
problem mappings, consistent terminal and accepted operations, failed required
readiness dependencies, optional dependency failure and bounded additive
response extensions. Expected rejection remains a successful expectation-schema
check, not an inverted process failure.

| Input | SHA-256 |
| --- | --- |
| `contracts/public/v1/common.schema.json` | `8e0f3c2844abafe8d2c878c9bc300834e92ea1c506055c8577e5f910c8532dfa` |
| `contracts/validation/common-v1.cases.json` | `2842b75b9d5c8dc9ddc765959a28ba5f99f7a4a23e5e7de8cf77fdacf4324a9e` |
| `contracts/validation/common-v1.expectations.schema.json` | `c36eb7ec3a242df5c4c83299c0908d8524a67b8ef495ef101f18b0fa18177b32` |
| `contracts/validation/pyproject.toml` | `967b5ddfdad0d02748762941d85e950e82a688fd1d4ab1df46bda6ac4b0f562a` |
| `contracts/validation/uv.lock` | `eb623efc79fba992bf40434b08a388be5136693a719fe02b6659a39b9c112268` |
| `contracts/validation/README.md` | `4708740e730eaaa47bc815acfdc4829f6da140538614037032b42cdf3739664f` |
| `docs/requirements/common-contract.md` | `8fcf53bf1cd39114fa53c6caf33ce92f84d69decbafc00fdfb29bd7cbbc2d5a7` |

Sixteen isolated metadata/oracle mutations each produced validation exit 1:
missing, duplicate, unknown or inconsistent inventory metadata, altered
expectation polarity and substituted values cannot pass the finite inventory.
Missing local references in both positive and negative-`not` contexts separately
produced infrastructure exit 1. Restoring the original manifest passed again
with exit 0. Hash guards confirmed that these diagnostics changed none of the
three final schema/fixture inputs. The task-owned diagnostic copies and initial
probe were removed after verification.

The common library has 28 named definitions and 52 internal fragment references.
The expectation schema uses only the adjacent versioned common schema; its
fixed case identities, target names, polarity and reference branches are checked
against the manifest. Neither file defines a base identifier or uses a dynamic
or remote application-schema reference. The definition library fails closed
when used without selecting a named definition.

The proposed checkout has 51 files, including 39 Markdown files and no Rust or
Python source. Local link/fragment review checked 276 targets; eight distinct
external links were identified separately. These bounded documentation checks
do not become a private dependency of the supported public contract command.

Independent semantic review reconciled all 248 case identities, target names,
expected outcomes and schema branches. A separate JSON inspection found no
duplicate keys or non-finite tokens in the committed inputs. Review corrected
overly restrictive response extension names and closed nested response objects,
while preserving strict requests and contradictory-outcome rejection. It also
confirmed that this control-message profile cannot silently truncate the
separately required compiled-context envelope. Runtime contract composition
remains explicit in the owning requirements.

The final diff and commit metadata retain existing rights and public boundaries.
The locked tools use public upstream distribution records. The supported checks
need neither another source checkout nor undisclosed context. `git diff --cached
--check` passed for all 15 changed files, including the new contracts and fixtures.
The following plan record will identify the exact source commit.

Raw JSON parsing, HTTP responses, authentication, authorization and runtime
effects remain unexecuted requirements. No Rust/Python product source is added
by this contract checkpoint; its declarative checks do not claim source coverage
or later service/platform qualification.
