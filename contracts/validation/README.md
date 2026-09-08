# Public contract validation

This project prepares upstream validators for the versioned public contracts.
It contains tool configuration and declarative fixtures, with no first-party
Python implementation. The engine contract runner will consume the same reviewed
contracts and oracles when its implementation is available.

The [common contract](../../docs/requirements/common-contract.md),
[acquisition contract](../../docs/requirements/acquisition-contract.md),
[observation contract](../../docs/requirements/observation-contract.md),
[candidate review contract](../../docs/requirements/candidate-review-contract.md),
[financial contract](../../docs/requirements/financial-records.md),
[context contract](../../docs/requirements/context-contract.md) and
[lifecycle contract](../../docs/requirements/lifecycle-contract.md) own wire
meaning. Each has a versioned schema, independently chosen parsed-value cases
and a fixed expectation schema:

| Family | Contract schema | Case manifest | Expectation schema |
| --- | --- | --- | --- |
| Common | [Shared definitions](../public/v1/common.schema.json) | [Common cases](common-v1.cases.json) | [Common expectations](common-v1.expectations.schema.json) |
| Acquisition | [Upload and capture definitions](../public/v1/acquisition.schema.json) | [Acquisition cases](acquisition-v1.cases.json) | [Acquisition expectations](acquisition-v1.expectations.schema.json) |
| Observation | [Grounding and source definitions](../public/v1/observation.schema.json) | [Observation cases](observation-v1.cases.json) | [Observation expectations](observation-v1.expectations.schema.json) |
| Candidate review | [Candidate and review definitions](../public/v1/candidate-review.schema.json) | [Candidate review cases](candidate-review-v1.cases.json) | [Candidate review expectations](candidate-review-v1.expectations.schema.json) |
| Financial records | [Expense, report and export definitions](../public/v1/financial-records.schema.json) | [Financial cases](financial-records-v1.cases.json) | [Financial expectations](financial-records-v1.expectations.schema.json) |
| Context | [Build, evaluation and attestation definitions](../public/v1/context.schema.json) | [Context cases](context-v1.cases.json) | [Context expectations](context-v1.expectations.schema.json) |
| Lifecycle | [Public operation/lifecycle definitions](../public/v1/lifecycle.schema.json) and [internal continuity definitions](../internal/v1/lifecycle-continuity.schema.json) | [Lifecycle cases](lifecycle-v1.cases.json) | [Lifecycle expectations](lifecycle-v1.expectations.schema.json) |

The commands select the [common entry](../common-v1.fixtures.schema.json),
[acquisition entry](../acquisition-v1.fixtures.schema.json),
[observation entry](../observation-v1.fixtures.schema.json),
[candidate review entry](../candidate-review-v1.fixtures.schema.json),
[financial entry](../financial-records-v1.fixtures.schema.json),
[context entry](../context-v1.fixtures.schema.json) and
[lifecycle entry](../lifecycle-v1.fixtures.schema.json) at the contract
directory's root. Each entry references exactly its own expectation schema;
swapping fixture families must fail. The entries contain no copied domain
definitions or expected values.

Lifecycle is one family with two explicit target namespaces:
public.<definition> maps exactly to
../public/v1/lifecycle.schema.json#/$defs/<definition>, and
internal.<definition> maps exactly to
../internal/v1/lifecycle-continuity.schema.json#/$defs/<definition> from the
expectation schema. No fallback, arbitrary URI or same-name ambiguity is allowed.
Its fixed ordered oracle records strict boolean acceptance independently of
schema execution. Internal references may import ../../public/v1 definitions;
public libraries never import internal definitions. Every transitive reference
remains within this checkout's contracts directory.

The separate [signing-vector fixture](context-v1.signing-vectors.json) and
[vector schema](../context-v1.signing-vectors.schema.json) retain a finite exact
inventory of published-key test messages and expected primitive outcomes. Their
parsed check validates declarative structure; it does not execute a signer,
compute canonical bytes or establish current issuer trust. The context owner
defines the separate actual vector verification and later runtime qualifications.

These parsed-value checks do not implement an HTTP service, raw JSON parser,
authentication, command admission or sealed storage. Each owner separately
identifies semantic scenarios that need the later runtime suites.

## Preparation

Use exactly uv 0.12.10 and CPython 3.14.7 for this validator profile. The
project and committed lock select check-jsonschema 0.38.0, jsonschema 4.26.0
and their complete transitive dependency set. This environment is separate from
any future processing-service environment. Do not install optional parsers or
ambient plugins into it.

From this repository's root, prepare the isolated project explicitly:

```sh
uv sync --project contracts/validation --locked --python 3.14.7
```

On workstations with a required build-output router, first run its diagnostic
and route all preparation/check commands through it. Select a dedicated approved
environment and artifact directory for contract validation. Do not change host
defaults or share the processing service's environment. Normal use requires only
this checkout and the documented upstream tools.

Verify exact synchronization before running the checks:

```sh
uv lock --project contracts/validation --check --offline
uv sync --project contracts/validation --check --locked --offline
```

## Checks

Run all nine commands from this repository's root after preparation. Every
command must exit zero. Explicit file arguments prevent an empty glob from
selecting no fixtures. Together with the two synchronization checks above,
these are eleven required verification commands. The metaschema command names
all 23 schema files; seven commands each select one exact family and the final
command checks the separate literal signing-vector structure.

```sh
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --check-metaschema --force-filetype json \
  --regex-variant default --no-cache \
  contracts/common-v1.fixtures.schema.json \
  contracts/acquisition-v1.fixtures.schema.json \
  contracts/observation-v1.fixtures.schema.json \
  contracts/candidate-review-v1.fixtures.schema.json \
  contracts/public/v1/common.schema.json \
  contracts/validation/common-v1.expectations.schema.json \
  contracts/public/v1/acquisition.schema.json \
  contracts/validation/acquisition-v1.expectations.schema.json \
  contracts/public/v1/observation.schema.json \
  contracts/validation/observation-v1.expectations.schema.json \
  contracts/public/v1/candidate-review.schema.json \
  contracts/validation/candidate-review-v1.expectations.schema.json \
  contracts/financial-records-v1.fixtures.schema.json \
  contracts/public/v1/financial-records.schema.json \
  contracts/validation/financial-records-v1.expectations.schema.json \
  contracts/context-v1.fixtures.schema.json \
  contracts/public/v1/context.schema.json \
  contracts/validation/context-v1.expectations.schema.json \
  contracts/context-v1.signing-vectors.schema.json \
  contracts/lifecycle-v1.fixtures.schema.json \
  contracts/public/v1/lifecycle.schema.json \
  contracts/internal/v1/lifecycle-continuity.schema.json \
  contracts/validation/lifecycle-v1.expectations.schema.json
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --schemafile contracts/common-v1.fixtures.schema.json \
  --force-filetype json --regex-variant default --no-cache \
  contracts/validation/common-v1.cases.json
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --schemafile contracts/acquisition-v1.fixtures.schema.json \
  --force-filetype json --regex-variant default --no-cache \
  contracts/validation/acquisition-v1.cases.json
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --schemafile contracts/observation-v1.fixtures.schema.json \
  --force-filetype json --regex-variant default --no-cache \
  contracts/validation/observation-v1.cases.json
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --schemafile contracts/candidate-review-v1.fixtures.schema.json \
  --force-filetype json --regex-variant default --no-cache \
  contracts/validation/candidate-review-v1.cases.json
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --schemafile contracts/financial-records-v1.fixtures.schema.json \
  --force-filetype json --regex-variant default --no-cache \
  contracts/validation/financial-records-v1.cases.json
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --schemafile contracts/context-v1.fixtures.schema.json \
  --force-filetype json --regex-variant default --no-cache \
  contracts/validation/context-v1.cases.json
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --schemafile contracts/lifecycle-v1.fixtures.schema.json \
  --force-filetype json --regex-variant default --no-cache \
  contracts/validation/lifecycle-v1.cases.json
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --schemafile contracts/context-v1.signing-vectors.schema.json \
  --force-filetype json --regex-variant default --no-cache \
  contracts/validation/context-v1.signing-vectors.json
```

The fixture manifest records independently chosen expected acceptance and
rejection. The expectation schema applies the real contract to accepted values
and its logical `not` to rejected values. All cases therefore have to pass this
expectation check. Do not invert the CLI's exit status for a negative fixture:
schema-resolution and other infrastructure errors also produce a nonzero exit.
The finite inventory must reject missing, repeated and unknown case identities.

Keep default format checking enabled and use the declared ECMAScript regex
behavior. The direct pin `rfc3986-validator==0.1.1` supplies the upstream
jsonschema URI and URI-reference implementation. The effective CLI registry
must contain `date`, `date-time`, `uri`, `uri-reference` and `regex`, including
the check-jsonschema date-time override and default ECMAScript regex checker.
An absent optional format checker silently accepts values; a successful generic
validation is insufficient proof that the required checker is available.
Freeze the actual installed package, interpreter and callback identities before
qualification and confirm they remain unchanged afterward. Qualify newly used
formats when the schema or metaschema graph changes.

Do not fill defaults, transform data, substitute validators or override
base URIs. The schema files omit `$id`; versioned repository paths and recorded
digests identify them. Common definitions use internal fragments. Other domain
definitions additionally reference the reviewed adjacent schemas. The expectation
schemas use reviewed relative references to their owning versioned schemas.
No nested identifier or dynamic/remote reference may change that resolution
scope. Review the complete reference graph when changing any schema;
successful metaschema validation alone does not prove that every reference
resolves.

This pinned tool has a
[nested local-reference limitation](https://github.com/python-jsonschema/check-jsonschema/issues/640).
Starting at the named entry makes the first reference descend into the contract
tree, so subsequent relative resolution stays within that namespace. Invoke
these entries instead of starting directly from a nested expectation schema.
Every transitive application-schema reference must remain within `contracts/`.
Changes to that graph need actual resolution checks, swapped-family and missing
reference failures, and relocation verification. No extra base URI or schema
identifier is injected to make a fixture pass.

These checks need no network resource: standard Draft 2020-12 metaschemas are
bundled with the prepared dependencies, and all other references are local.
The `uv --offline` option controls dependency access; `--no-cache` disables the
validator's cache. Neither option is a network sandbox. Do not use them to claim
that an unreviewed remote reference is blocked.

The upstream parser validates already parsed values. Raw duplicate keys,
non-finite tokens, byte/depth admission, malformed encoding, transport headers,
current authorization and transactional effects need the owning runtime tests.
Their contract scenarios remain requirements until those tests execute. This
fixture pass cannot substitute for runtime or source-coverage evidence.

Freeze the actual complete machine inventory before qualification: 23 schemas,
seven literal case manifests, the separate signing-vector manifest, validator
project and lock (33 inputs). Freeze actual per-target/namespace/case counts and
input digests after independent authoring; no count or positive outcome may be
inferred from an earlier family. Run all eleven commands against those exact
bytes. Relocation must copy/re-hash every input into a path with spaces and
Unicode and exercise all seven family entries plus signing structure.

Independently review finite oracle-integrity mutations for metadata, missing or
duplicated identities, order, strict booleans, target substitution across the
public/internal namespace, and positive/negative value replacement in both
namespaces. Break actually consumed references under both positive refs and
negative not branches; require their missing-reference diagnostics, never a
successful negation. Remove each actually consumed application library and
observe the exact missing-path diagnostic. All 42 off-diagonal family pairings
must fail ordinary validation on the complete manifests. Restore and re-hash
every input and rerun positive guards. Tool/timeout/environment or unrelated
schema-resolution failure is not a negative value oracle.

The current complete protocol has 93 commands: eleven direct checks, eight
relocated checks, eighteen oracle mutations, four consumed broken references,
eight consumed missing libraries, 42 off-diagonal pairings and two restored
positive guards. A changed primary validator profile requires this complete
protocol afresh on the newly frozen inputs; results from an older profile cannot
be combined into a current pass.

The URI regression inventory contains sixteen variants in each of staging
grant, acquisition allocation response and lifecycle allocation response:
48 complete values with 21 positive and 27 negative expectations. Preserve
every earlier case and expectation. Independently freeze the literal outcomes
before checking them, then validate every new complete value directly against
its actual owning definition. Every negative must report the URI-format error
at its actual URL field. One invalid URI per target must additionally accept
with only URI checking disabled, as a separately labeled diagnostic control;
that acceptance is not conformance. These controls do not replace the complete
manifest checks. Valid generic URI syntax for userinfo and fragments does not
override the acquisition owner's runtime client restrictions, origin checks or
current authorization requirements.

The lifecycle owner separately requires current authority and disclosure,
exact JCS bytes/hashes and cross-record joins, complete actual serialization,
finite clocks, cancellation/handoff races, retained recognition after receipt
expiry, independent journal crash/abort/restore proof, real sealed delivery,
immutable feedback attribution and actual multi-batch repair/service coverage.
These are consuming runtime obligations, not results of parsed-value checks.

Tool packages and their notices are obtained from the locked upstream artifacts;
their sources and bundled example schemas are not copied into these contracts.
No upstream built-in application schema is selected by the commands above.

The CLI behavior and validation options are documented by
[check-jsonschema](https://check-jsonschema.readthedocs.io/en/latest/usage.html).
The contract uses the
[JSON Schema 2020-12 validation vocabulary](https://json-schema.org/draft/2020-12/json-schema-validation).
