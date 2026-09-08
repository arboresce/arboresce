# Public contract validation

This project prepares upstream validators for the versioned public contracts.
It contains tool configuration and declarative fixtures, with no first-party
Python implementation. The engine contract runner will consume the same reviewed
contracts and oracles when its implementation is available.

The [common contract](../../docs/requirements/common-contract.md),
[acquisition contract](../../docs/requirements/acquisition-contract.md) and
[observation contract](../../docs/requirements/observation-contract.md) own wire
meaning. Each has a versioned schema, independently chosen parsed-value cases
and a fixed expectation schema:

| Family | Contract schema | Case manifest | Expectation schema |
| --- | --- | --- | --- |
| Common | [Shared definitions](../public/v1/common.schema.json) | [Common cases](common-v1.cases.json) | [Common expectations](common-v1.expectations.schema.json) |
| Acquisition | [Upload and capture definitions](../public/v1/acquisition.schema.json) | [Acquisition cases](acquisition-v1.cases.json) | [Acquisition expectations](acquisition-v1.expectations.schema.json) |
| Observation | [Grounding and source definitions](../public/v1/observation.schema.json) | [Observation cases](observation-v1.cases.json) | [Observation expectations](observation-v1.expectations.schema.json) |

The commands select the [common entry](../common-v1.fixtures.schema.json),
[acquisition entry](../acquisition-v1.fixtures.schema.json) and
[observation entry](../observation-v1.fixtures.schema.json) at the contract
directory's root. Each entry references exactly its own expectation schema;
swapping fixture families must fail. The entries contain no copied domain
definitions or expected values.

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

Run all four commands from this repository's root after preparation. Every
command must exit zero. Explicit file arguments prevent an empty glob from
selecting no fixtures.

```sh
uv run --project contracts/validation --no-sync --offline \
  check-jsonschema --check-metaschema --force-filetype json \
  --regex-variant default --no-cache \
  contracts/common-v1.fixtures.schema.json \
  contracts/acquisition-v1.fixtures.schema.json \
  contracts/observation-v1.fixtures.schema.json \
  contracts/public/v1/common.schema.json \
  contracts/validation/common-v1.expectations.schema.json \
  contracts/public/v1/acquisition.schema.json \
  contracts/validation/acquisition-v1.expectations.schema.json \
  contracts/public/v1/observation.schema.json \
  contracts/validation/observation-v1.expectations.schema.json
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
```

The fixture manifest records independently chosen expected acceptance and
rejection. The expectation schema applies the real contract to accepted values
and its logical `not` to rejected values. All cases therefore have to pass this
expectation check. Do not invert the CLI's exit status for a negative fixture:
schema-resolution and other infrastructure errors also produce a nonzero exit.
The finite inventory must reject missing, repeated and unknown case identities.

Keep default format checking enabled and use the declared ECMAScript regex
behavior. Do not fill defaults, transform data, substitute validators or override
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

Tool packages and their notices are obtained from the locked upstream artifacts;
their sources and bundled example schemas are not copied into these contracts.
No upstream built-in application schema is selected by the commands above.

The CLI behavior and validation options are documented by
[check-jsonschema](https://check-jsonschema.readthedocs.io/en/latest/usage.html).
The contract uses the
[JSON Schema 2020-12 validation vocabulary](https://json-schema.org/draft/2020-12/json-schema-validation).
