# Local validator foundation evidence

This record covers the local contract validator and deterministic development
suite. It does not establish full-corpus operational adoption, the selection
contract, cursor runtime, service behavior or another platform profile.
The [rolling plan](open-cli-mvp-v1-rcld.md) retains those dependent gates.

## Measured source profile

The complete suite executed on 2026-09-10 with 293 passing tests in 46.22 seconds.
Complete collection, all five real CLI read barriers, 96 actual child processes
and 97 raw coverage profiles were independently reconciled. The direct prepared
pytest and coverage commands are documented in the
[development procedure](../../contracts/validation/README.md#development-tests).
Standard `coverage combine --keep`, `coverage json` and `coverage report` all
exited zero. The independently reviewed combined database preserves every raw
file/context/arc contribution; byte-identical profiles remain retained even when
the upstream combiner deduplicates their data.

The qualified host profile is macOS ARM64 with CPython 3.14.7, uv 0.12.10,
check-jsonschema 0.38.0, jsonschema 4.26.0 and rfc3986-validator 0.1.1. Development
tools are pytest 9.1.1, coverage 7.16.0 using sys.monitoring branch collection,
Hypothesis 6.167.1, Ruff 0.16.6 and Pyright 1.1.411 with Node 26.8.1. The project
and lock below bind the remaining dependency versions. Normal dependencies and
development tools use separate environments.

Every applicable maintained-source line and branch gate uses raw integers:
`total > 0 and 10 * covered > 9 * total`. Exactly 90 percent fails. The following
counts come from the accepted unchanged measured source, not from a new test run
performed merely to write this document.

| Maintained file, relative to contracts/validation | Covered lines / total | Covered branches / total |
| --- | --- | --- |
| `src/arboresce_contract_validation/__init__.py` | N/A: empty initializer | N/A: no executable branches |
| `src/arboresce_contract_validation/cli.py` | 302 / 310 | 82 / 86 |
| `tests/conftest.py` | 21 / 21 | N/A: no executable branches |
| `tests/process_observation/observe_cli.py` | 256 / 272 | 116 / 128 |
| `tests/support.py` | 496 / 509 | 133 / 144 |

Maintained-source totals are 1,075/1,112 lines and 331/358 branches. The eight
assertion-only test modules remain explicitly inventoried as tests; their bodies
do not inflate these denominators. The empty initializer is not fictional
100-percent coverage. Nine API declaration files are separately inventoried
non-executable source and require strict static and pinned-API behavioral checks.
Formatting, lint and strict typing passed for this measured profile.

## Deterministic behavior and applicability

The committed fixtures cover valid/invalid JSON values, exact resource limits,
file replacement and descriptor races, missing filesystem capabilities, lazy
instance parsing, closed reference retrieval, malformed and missing references,
upstream parser/validator/reporting compatibility, and real CLI process barriers.
Independent literal and behavioral reviews remain separate from coverage
arithmetic. A fixed property profile and seed, controlled locale/timezone,
explicit plugin collection and fresh external output paths make the suite
reproducible without production services or credentials.

The measured source, tests, fixtures, configuration and declaration files below
are byte-identical in this foundation. Documentation has been scoped to preserve
the existing seven-family, 23-schema upstream corpus catalogue. The local
component's development suite does not consume the pending selection schema or
cursor fixtures. Any change to bound executable, fixture, configuration,
dependency or execution-profile inputs requires renewed affected qualification;
this record cannot be transferred to an incompatible source or platform.

The dependent checkpoint must still qualify full operational command adoption,
complete configuration and URI diagnostics, current guards, complete family and
reference/relocation behavior, native cursor checks and final consumers. Earlier
full-family timeouts remain failures until fresh complete checks pass within
the unchanged bounds.

## Source and fixture fingerprints

Paths are relative to `contracts/validation/`. SHA-256 fingerprints identify the
exact component inventory; this document and contributor guidance are not
executable coverage inputs.

| Path | Bytes | SHA-256 |
| --- | --- | --- |
| `pyproject.toml` | 928 | `d80f289b3a030d35bab7c92829b92cfadfd2df4037eb576ca3d78acea2d450bd` |
| `src/arboresce_contract_validation/__init__.py` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `src/arboresce_contract_validation/cli.py` | 18033 | `7382bae11eabc6b2ae4eb1ee276cbab897ffcc32a27bd68bb582560a1f06297b` |
| `tests/conftest.py` | 1882 | `c870554cdaca92d3d46c981a59510155691270bad376bd313ab656ff950f9a4f` |
| `tests/coverage.ini` | 232 | `4b1ede36fad4be2c068611e25db5d0e9c3fe45d0ef07130dc2f54d59ce4570d3` |
| `tests/fixtures/boundaries.json` | 131939 | `2689daba76beb0c27cb94d03153f60dac821f51b3a76cc4b178905d2f6e62415` |
| `tests/fixtures/coverage-report.json` | 2157 | `b8256daf1a1d665c635f2fff54cc5e63345abe026cd2e71e148a225c9a8fb8a8` |
| `tests/fixtures/process-observation-guards.json` | 21737 | `aa9bebe212d66b3eb1b43e56893bb85977bc0b7726c9c7144258b3794b6b9487` |
| `tests/fixtures/support-guards.json` | 11948 | `a3930d718b5b24a1745aee19f98d1a6f8b4d3ed357f888c8c427f76961802243` |
| `tests/fixtures/upstream-api.json` | 4508 | `1d7be69310742837b18b5c5f133234bb01de4062665686392e5c28f3053b39bd` |
| `tests/fixtures/validation.json` | 7173 | `9fb8ad909256803f619959c94e2a649aea11db329fe560f012599758b374a4af` |
| `tests/process_observation/observe_cli.py` | 17338 | `665a88de86def8d7029da748c7e36d3b74296344663bcdbc632de35835bb95c4` |
| `tests/support.py` | 31837 | `5c850e1421ebc0a333f27582fcab611ba13ca46d472ee03e15c80096883e32b3` |
| `tests/test_cli.py` | 8822 | `cc01228638c9a3e7ada7085776511f30bcbb3fa6da37ce9b9cdd133a3e4046bc` |
| `tests/test_discovery.py` | 2919 | `9b324e2fefd68537fddbae4f9b40df49dce9e3602edbf2db1e003e8c0cd8edde` |
| `tests/test_properties.py` | 5188 | `8d044b6df47fe0d15d401121e43812cc148aae9853f040eab067fef66f4db6a1` |
| `tests/test_races.py` | 1254 | `8948710800534fcbe405363fa1cd6ac5ab021a5ec4890223677f1e787b11cc32` |
| `tests/test_references.py` | 7545 | `77b6218284a25d9ec4b305532434b00d67e9326bd657b72d9e6827c90d9d098c` |
| `tests/test_snapshot.py` | 7725 | `1562c12502eab92603f4e7df76e377f5304ce30face31f81a59c59ae71eb201c` |
| `tests/test_support.py` | 37957 | `f85072d7d7c776fd76d58aae0d9e1ea2b5bde537402cd23363b077cafbe2ca47` |
| `tests/test_validation.py` | 12020 | `13616ea3635a28425500adbc0adcac12f1a4c9ef2fd9bb73eaea2f7ead09b5e6` |
| `typings/check_jsonschema/__init__.pyi` | 84 | `589ac722610ac191be9a49fe5550e6ec6c0d6fa57f779edf04cd489c7737af04` |
| `typings/check_jsonschema/checker.pyi` | 724 | `18b68e52c80bfe9ee50e48c3e214cf157c770d72e162738423ecf211e6792f62` |
| `typings/check_jsonschema/formats/__init__.pyi` | 429 | `27065407f0cc44e3c479d703ee4fb4c34318aff06363a6cc6997774b5063f614` |
| `typings/check_jsonschema/instance_loader.pyi` | 337 | `073de8d5e4eb763ef2fae460f418bf646e3c048efe9dd111c8fca08227353a5e` |
| `typings/check_jsonschema/parsers/__init__.pyi` | 386 | `4b8a15672b864a3ba6f2187d1db779cff3d4f557ea0a6175d53816bc246e174b` |
| `typings/check_jsonschema/regex_variants.pyi` | 335 | `4a3ed53bc9e20061429c3031ca232c7ad08809c139647f9694a3ea7b3b1fe65c` |
| `typings/check_jsonschema/reporter.pyi` | 560 | `b5c3432c488025dc3932e3fadcd830bd79d14c011465d7e2fd3190664e90752c` |
| `typings/check_jsonschema/result.pyi` | 108 | `1ec602ee9fb85d2df35a40ff07f4fa39636f9eb1fd71e18dc89588d6a1449387` |
| `typings/check_jsonschema/schema_loader/__init__.pyi` | 921 | `3476e6b41851dbb495e15a471237009cf48d27d40a2c2b0435bff974ab3eb48e` |
| `uv.lock` | 87406 | `52afa3002a1ec4cb8eb92f2f2d5656ca2b52f540c199737c9072d9159a14434e` |
