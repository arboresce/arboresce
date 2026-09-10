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
uv sync --project contracts/validation --locked --no-dev --python 3.14.7
```

On workstations with a required build-output router, first run its diagnostic
and route all preparation/check commands through it. Select a dedicated approved
environment and artifact directory for contract validation. Do not change host
defaults or share the processing service's environment. Normal use requires only
this checkout and the documented upstream tools.

Verify exact synchronization before running the checks:

```sh
uv lock --project contracts/validation --check --offline
uv sync --project contracts/validation --check --locked --no-dev --offline
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

## Local validator component

The repository also supplies a tested local validator at
`contracts/validation/src/arboresce_contract_validation/cli.py`. The corpus
commands above retain their upstream entrypoints. Full-corpus adoption of this
component and its operational timeout qualification remain pending.

From the repository root, the component accepts one local schema and one or
more explicit local JSON instance paths beneath `contracts/`:

```text
uv run --project contracts/validation --no-sync --offline python -I -B \
  contracts/validation/src/arboresce_contract_validation/cli.py \
  --schemafile SCHEMA_PATH --force-filetype json --regex-variant default \
  --no-cache INSTANCE_PATH [INSTANCE_PATH ...]
```

Replace the uppercase arguments with explicit local paths. Optional output
controls are `--output-format text|json`, `--traceback-mode short|full`,
`--verbose` and `--quiet`. Success exits zero. Nonzero results include invalid
values, usage and infrastructure failures; a nonzero exit alone never proves
a negative fixture. Metaschema and URI diagnostic checks retain the upstream
commands and their separate qualification. The deterministic development suite
exercises real instances of this component without requiring a product service.

The local command loads the current contract schemas into a fresh invocation
registry, retaining its crawled result. Each resource has its canonical file URI
and an entry-relative alias. This avoids repeated traversal of the complete
resource graph while preserving the upstream resolver, fragments and effective
format/regex callbacks. Invoke the named entries; every transitive application
reference remains within `contracts/`. No extra base URI, schema identifier,
private resolver state or replacement keyword is used to make a fixture pass.

One descriptor reader admits entry schemas, referenced schemas and instances.
It rejects symlinks, special files and paths outside the current contract root.
It opens without following links or blocking on special files, checks regular
file identity before payload reads, and rejects growth, shrinkage, replacement
or parent substitution. Stable snapshots allow at most 64 MiB of payload per
file, 1,280 unique files and 512 MiB of requested reads per invocation. The
one-byte EOF probe counts against requested reads, including for an exactly
64 MiB payload. Discovery admits at most 1,280 directories and 2,560 total
directory entries, counting entries before sorting. The contract root is depth
zero; descent beyond depth 64 fails before opening the next directory. Required
filesystem capabilities are checked before opening the invocation root; their
absence identifies that root. File-specific admission failures identify the
actual file. Every descriptor closes on success or failure.

Instance arguments receive ordered descriptor-only preflight before parsing.
Preflight reads no payload and retains no snapshot. A statically missing or
unreadable later argument therefore fails before any instance is parsed. Each
lazy first read reopens and fully admits the actual file; disappearance after
preflight remains a separate first-read error. Instances parse before the entry
schema is loaded. If all instances fail to parse, the schema is never loaded.
For a parsed instance, upstream entry metaschema validation precedes reference
preloading and value validation.

Retrieval consumes only already admitted immutable snapshots. It never opens a
file, downloads a resource or consults another invocation. An interpretation
error while preloading a referenced schema discards the partial registry and
uses a fresh lazy registry over those same snapshots. Unconsumed malformed
references do not become new schema failures; consuming them preserves actual
upstream errors, including under `not`. Admission failures, exhausted deadlines,
memory exhaustion and process-control exceptions never select that fallback.
Operational adoption of the committed corpus must separately prove the optimized path. Preserve complete error
order and text/JSON reports; infrastructure failures cannot become ordinary
negative-value acceptance. Changes require actual resolution, swapped-family,
missing-reference and relocation checks.

The loader/checker integration is explicitly pinned to check-jsonschema 0.38.0.
Dependency changes require source and behavioral requalification. Preserve its
ECMAScript `pattern`/`patternProperties` behavior and the pinned jsonschema
additional-property discovery behavior; these do not use identical regex rules
for every Unicode property name. Independently reviewed regression literals
record that compatibility boundary.

The declaration subset in `typings/check_jsonschema/` describes only the
check-jsonschema 0.38.0 interfaces consumed by this project. Pyright discovers
it through the default project-relative `typings` directory. Keep strict
diagnostics enabled. The default loader returns a concrete jsonschema validator
with an instance `format_checker`; the upstream `Validator` protocol omits that
attribute. A local type-only protocol records this pinned boundary. Integration
tests must verify the actual concrete class, schema and format-checker identity
as well as the effective keyword functions and semantic outcomes. A type cast
does not establish those results. Review the declarations and coupling tests
whenever either dependency changes. These nine declaration files are static,
non-executable artifacts; their inventory is separate from executable coverage.

Tests keep warnings-as-errors enabled. The pinned check-jsonschema release
accesses Click 8.5's deprecated `LazyFile` compatibility alias during its first
import. Test initialization positively asserts that one exact upstream warning,
including its category and source location, and retains the actual diagnostic
in the suite receipt. Missing, duplicate or unexpected warnings fail. Review this
assertion when either dependency changes; do not add an ignore filter.


## Development tests

Use the repository-owned pytest tests and explicit coverage collection below. Prepare tools and external resources before verification; apply required workstation build-output routing to the commands. These commands exercise deterministic local fixtures and do not establish unrelated runtime or platform qualification.

The fixture set includes exact upstream API signatures and real validator behavior, source-copy integrity, measurement-file integrity, and pipe barriers around actual CLI reads. Guard tests mutate disposable files at explicit I/O boundaries and retain original bytes. Two prelaunch collision tests retain deliberate error receipts without creating a child; keep those receipts separate from actual child-process coverage. Constructed coverage databases test the integrity checks and never enter measured coverage aggregation. Reusable fixture and observation code is maintained source and must satisfy the same strict coverage and type-checking requirements.

Set `PYTHONDONTWRITEBYTECODE=1` when running Pyright with the prepared Python interpreter. This also applies to Pyright's interpreter-discovery child and preserves the qualified environment's file inventory.

Prepare the dev dependencies separately in a dedicated external environment, using the README's exact uv 0.12.10 and Python 3.14.7 selections:

```sh
UV_PROJECT_ENVIRONMENT="$REGISTRY_DEV_ENV" uv sync \
  --project contracts/validation --locked --group dev --python 3.14.7
```

Run formatting, lint and strict typing from this project's directory before tests. Use the locked Ruff 0.16.6 and Pyright 1.1.411 packages with a separately prepared Node 26.8.1 executable. Prepare and qualify that exact Node distribution and its native dependencies under [dependency qualification](../../docs/requirements/dependency-qualification.md); verification never installs Node, changes a host default or obtains a different Pyright version.

For the prepared POSIX Python 3.14 environment, set REGISTRY_PROJECT to the absolute contracts/validation directory, REGISTRY_PYTHON to REGISTRY_DEV_ENV/bin/python, REGISTRY_NODE to the absolute prepared Node executable, and REGISTRY_ARTIFACTS to an approved external output parent. The bundled entrypoint below belongs to the pinned Pyright wheel. A different installation layout needs its actual entrypoint identified during preparation.

```sh
umask 077
REGISTRY_STATIC=$(mktemp -d "$REGISTRY_ARTIFACTS/registry-static.XXXXXXXX")
mkdir "$REGISTRY_STATIC/home" "$REGISTRY_STATIC/tmp" "$REGISTRY_STATIC/cache"
REGISTRY_PYRIGHT_JS="$REGISTRY_DEV_ENV/lib/python3.14/site-packages/pyright/dist/index.js"
env -i PATH=/usr/bin:/bin LANG=C LC_ALL=C TZ=UTC \
  HOME="$REGISTRY_STATIC/home" TMPDIR="$REGISTRY_STATIC/tmp" \
  XDG_CACHE_HOME="$REGISTRY_STATIC/cache" PYTHONDONTWRITEBYTECODE=1 \
  /bin/sh -c '
    set -euC
    cd "$1"
    test "$("$2" --version)" = "ruff 0.16.6"
    test "$("$3" --version)" = "v26.8.1"
    test "$("$3" "$4" --version)" = "pyright 1.1.411"
    "$2" format --check --no-cache . >"$6/ruff-format.stdout" 2>"$6/ruff-format.stderr"
    "$2" check --no-cache . >"$6/ruff-check.stdout" 2>"$6/ruff-check.stderr"
    "$3" "$4" --project "$1/pyproject.toml" --pythonpath "$5" --outputjson \
      >"$6/pyright.json" 2>"$6/pyright.stderr"
  ' registry-static "$REGISTRY_PROJECT" "$REGISTRY_DEV_ENV/bin/ruff" \
  "$REGISTRY_NODE" "$REGISTRY_PYRIGHT_JS" "$REGISTRY_PYTHON" "$REGISTRY_STATIC"
```

Keep the exit status and all outputs; a failed command stops this sequence. Ruff uses the owning project's configuration and import classification, with [format --check](https://docs.astral.sh/ruff/formatter/) leaving source unchanged. Pyright reads the strict include list and local declaration stubs from pyproject.toml; review its complete JSON diagnostics and summary, requiring zero errors, warnings and information. Its [documented flags](https://github.com/microsoft/pyright/blob/1.1.411/docs/command-line.md) select the project and prepared interpreter explicitly.

The [pinned Python wrapper](https://github.com/RobertCraigie/pyright-python/blob/v1.1.411/src/pyright/_utils.py) can resolve another version or install an npm package, and its Node launcher can fall back to nodeenv. This command enters the wheel's existing index.js directly through the selected Node executable. The closed environment omits wrapper overrides, NODE_OPTIONS, NODE_PATH, PYTHONPATH, PYTHONHOME and automatic coverage-startup settings. PYTHONDONTWRITEBYTECODE preserves the prepared Python inventory when Pyright discovers interpreter paths. Apply the repository's required output routing and prepared resource limits; these ordinary commands do not by themselves certify process profiles or runtime coverage.

Verification uses that environment's direct bin/python and the explicitly prepared GNU coreutils 9.11 timeout; it does not install, resolve or fetch tools. Retain pytest 9.1.1, coverage 7.16.0 and Hypothesis 6.167.1. REGISTRY_PROJECT is this checkout's absolute contracts/validation path, REGISTRY_PYTHON is the development environment's bin/python, REGISTRY_TIMEOUT is the qualified timeout executable, and REGISTRY_ARTIFACTS is an already approved external output parent. The selected suite requires at least 2 GiB available RAM capacity, 8 GiB disk capacity and 256 descriptors; these are preparation requirements, not a claimed hard RSS/disk quota.

Create a fresh owner-only run root and each directory explicitly, outside contracts and the source checkout:

```sh
umask 077
REGISTRY_RUN=$(mktemp -d "$REGISTRY_ARTIFACTS/registry-tests.XXXXXXXX")
mkdir "$REGISTRY_RUN/home" "$REGISTRY_RUN/tmp" "$REGISTRY_RUN/cache" \
  "$REGISTRY_RUN/hypothesis" "$REGISTRY_RUN/base" \
  "$REGISTRY_RUN/profiles" "$REGISTRY_RUN/reports"
```

The following selects all eight test modules, including the process observation guard fixtures in test_support.py. The real /bin/sh process records its own $$ and execs timeout with that same PID. The only transient prepared Python command inspects inherited limits and publishes the original monotonic deadline/current-UID encoding before timeout starts. There is no uv or other process left between timeout and the measured pytest process. Current support then checks the actual parent/group/session and retains every actual CLI child/profile.

```sh
cd "$REGISTRY_PROJECT/../.."
env -i PATH=/usr/bin:/bin LANG=C LC_ALL=C TZ=UTC \
  HOME="$REGISTRY_RUN/home" TMPDIR="$REGISTRY_RUN/tmp" \
  XDG_CACHE_HOME="$REGISTRY_RUN/cache" \
  HYPOTHESIS_STORAGE_DIRECTORY="$REGISTRY_RUN/hypothesis" \
  PYTEST_DISABLE_PLUGIN_AUTOLOAD=1 PYTHONDONTWRITEBYTECODE=1 \
  COVERAGE_DEBUG=core COVERAGE_DEBUG_FILE="$REGISTRY_RUN/reports/coverage-core.log" \
  ARBORESCE_TEST_SUITE_RECEIPT="$REGISTRY_RUN/reports/suite.json" \
  /bin/sh -c '
    set -euC
    registry_python=$1
    registry_timeout=$2
    registry_run=$3
    shift 3
    exec >"$registry_run/reports/stdout" 2>"$registry_run/reports/stderr"
    ulimit -S -c 0
    ulimit -H -c 0
    ulimit -S -f 131072
    ulimit -H -f 131072
    registry_binding=$("$registry_python" -I -S -B -c \
      "import os,resource,time; assert resource.getrlimit(resource.RLIMIT_FSIZE)==(134217728,134217728); assert resource.getrlimit(resource.RLIMIT_CORE)==(0,0); assert resource.getrlimit(resource.RLIMIT_NOFILE)[0]>=256; print(time.monotonic()+320.0, f\"0x{os.getuid():X}:0x0:0x0\")")
    ARBORESCE_TEST_DEADLINE=${registry_binding%% *}
    __CF_USER_TEXT_ENCODING=${registry_binding#* }
    ARBORESCE_TEST_TIMEOUT_PID=$$
    export ARBORESCE_TEST_DEADLINE ARBORESCE_TEST_TIMEOUT_PID __CF_USER_TEXT_ENCODING
    exec "$registry_timeout" --signal=TERM --kill-after=5s 300s \
      "$registry_python" "$@"
  ' registry-tests "$REGISTRY_PYTHON" "$REGISTRY_TIMEOUT" "$REGISTRY_RUN" \
  -I -B -m coverage run --rcfile "$REGISTRY_PROJECT/tests/coverage.ini" \
  --data-file "$REGISTRY_RUN/profiles/.coverage" --context registry-selected-sysmon-v1 \
  -m pytest -c "$REGISTRY_PROJECT/pyproject.toml" --rootdir "$REGISTRY_PROJECT" \
  -p no:cacheprovider -p _hypothesis_pytestplugin --hypothesis-seed=0 \
  --basetemp "$REGISTRY_RUN/base" --junitxml "$REGISTRY_RUN/reports/junit.xml" \
  -vv --trace-config \
  "$REGISTRY_PROJECT/tests/test_cli.py" "$REGISTRY_PROJECT/tests/test_discovery.py" \
  "$REGISTRY_PROJECT/tests/test_properties.py" "$REGISTRY_PROJECT/tests/test_races.py" \
  "$REGISTRY_PROJECT/tests/test_references.py" "$REGISTRY_PROJECT/tests/test_snapshot.py" \
  "$REGISTRY_PROJECT/tests/test_support.py" "$REGISTRY_PROJECT/tests/test_validation.py"
```

The shell uses noclobber for stdout/stderr and a fresh owner-only output tree. Test support exclusively creates the suite receipt. The shown shell profile uses 1024-byte file-limit units; the pre-exec checks require actual soft/hard 134217728-byte file limits, zero core limits and at least 256 descriptors. A different platform unit convention fails before pytest starts and needs an explicitly qualified command adjustment. The standard platform encoding is derived from the current UID. Keep these actual bindings; do not copy a historical host UID.

HYPOTHESIS_STORAGE_DIRECTORY is required even with database=None: Hypothesis also writes constants. Explicit fresh storage keeps that runtime state outside the checkout. No ambient import, coverage-startup, plugin or pytest-option hooks are admitted. Keep the current exact positive pytest.warns assertion for one Click LazyFile DeprecationWarning at check-jsonschema/cli/param_types.py:126; do not suppress or pre-import around it. Its actual diagnostic remains in the suite receipt. The observation child imports only standard-library code before runpy executes the owned CLI and does not import parent support/conftest.

Before measurement, perform complete collection with another newly created run root using the same bridge, changing only the published original window to 45.0, the timeout to 30s, and the final Python vector to the following (same closed environment, limits, encoding and exclusive outputs):

```text
-I -B -m pytest -c PROJECT/pyproject.toml --rootdir PROJECT
-p no:cacheprovider -p _hypothesis_pytestplugin --hypothesis-seed=0
--collect-only -q --trace-config --basetemp RUN/base
PROJECT/tests/test_cli.py PROJECT/tests/test_discovery.py
PROJECT/tests/test_properties.py PROJECT/tests/test_races.py
PROJECT/tests/test_references.py PROJECT/tests/test_snapshot.py
PROJECT/tests/test_support.py PROJECT/tests/test_validation.py
```

These preserve the reviewed 320s/45s original observation/reconciliation windows and the separate 300+5/30+5 timeout behavior. Never replace the original deadline with a per-barrier allowance. The shell is a real entered process, not a parenthesized shell subshell that borrows another process's $$; include its one transient clock/limit inspection process in the full launch inventory. The bridge does not independently reconcile a timed-out group's terminal status or certify raw profiles: preserve a failed timeout and all artifacts. Existing qualification observation/reconciliation remains required for a qualification claim.

Optional diagnosis may replace the explicit module list with PROJECT/tests/test_cli.py::test_process_barriers, which selects exactly the five added process rows. That subset is not the default or a complete checkpoint. Collect and reconcile exact node and child/profile identities for the current source and fixture inventory. A changed source, config or dependency profile requires a new run; do not merge incompatible results.

Use standard coverage combine --keep, json and report with the complete accepted current raw profile list, a fresh combined data path and the reviewed configuration. Run these reporting commands from the same canonical engine root. Parent and copied child profiles then use the same contracts/validation/src and contracts/validation/tests paths; no additional path aliases or coverage configuration changes are needed. Keep every child profile/core log and complete process receipt. Measure the maintained observation helper with support/production code; assertion-only test files do not inflate those denominators. Apply total>0 and 10*covered>9*total independently to every applicable file/module/package metric. Neither a five-node diagnostic nor a successful standard-tool command proves complete qualification or coverage.
