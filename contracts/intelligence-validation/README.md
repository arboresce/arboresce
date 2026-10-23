# Intelligence transport conformance

This project owns closed protobuf framing and fixed JCS record reads for the
[intelligence transport](../../docs/requirements/intelligence-transport-v1.md).
Its package is `arboresce_intelligence_validation`. Its own project, lock and
environment select protobuf 7.36.2 without adding that dependency to the
[primary contract validator](../validation/README.md).

`intelligence_wire.decode_frame` accepts a trusted message class from the
committed descriptor and exact immutable bytes. It checks complete framing
before the pinned pure Python runtime decodes. Failure returns the static
`WireFailure.FRAME` value without partial messages or input-bearing exceptions.
The message class is program configuration, never a user-selected schema.

The component enforces the 1-MiB frame cap, sixteen-message nesting ceiling,
shortest unsigned varints, declared field/type/enum inventory, UTF-8 and recursive
duplicate singular/oneof refusal. Repeated messages and explicit optional zero
retain their meanings. Empty messages, arbitrary bytes and integer maxima can
be framing-valid while failing complete semantic admission.

[Independent inputs](../../tests/contract/intelligence/v1/README.md) contain
fifty literal frames and two frame-cap recipes. Expected answers stay in thin
assertions. `tests/intelligence_support.py` is measured support. The six local
protobuf declaration files describe only consumed interfaces of the pinned
runtime; actual descriptor loading and decoding must verify their coupling.
The [generation contract](../internal/proto/README.md) owns reproducible inputs.

`record_codec.decode_record` selects one of fourteen fixed `RecordCodec` targets
and requires explicit `RecordLimits`. It accepts exact immutable UTF-8/JCS bytes
within the selected record/depth limits, validates the complete owning schema,
and checks own identity/preimage, intrinsic quote and profile-limit conditions.
The pinned RFC 8785 writer has a bounded output sink. Candidate and expense
identities retain their public envelope preimages and normalize only their
declared sets; financial preimages retain their separate 131,072-byte ceiling.
The transport codec has no source-JSON node quota. A success returns immutable
canonical bytes and their digest with a redacted representation; failures return
only `RecordFailure.RECORD`. Configuration is program-selected, never a schema URI.

The eighty independently admitted literal record cases, twelve explicit-limit
cases and six intake forms keep expected answers in thin assertions.
`tests/record_support.py` is measured support. Record admission does not establish
full request/response joins or grant authority.
Selected combined profile limits, request/response identities, clocks, exposure
history and cost joins remain separate required consumers.
Generated service bindings, authenticated workers/providers, resource containment,
canonical storage, recovery and actual processing retain their qualification
gates. This component does not establish a running RPC or a usable product journey.

## Preparation and static checks

Select uv 0.12.10, CPython 3.14.7 and a dedicated external environment. Apply
repository output routing and its diagnostic before preparation or verification.
Run from the engine root with the chosen `UV_PROJECT_ENVIRONMENT`:

```sh
uv sync --project contracts/intelligence-validation --locked --all-groups --offline --python 3.14.7
uv lock --project contracts/intelligence-validation --check --offline
uv sync --project contracts/intelligence-validation --check --locked --all-groups --offline
uv run --project contracts/intelligence-validation --no-sync --offline \
  ruff format --check --no-cache contracts/intelligence-validation
uv run --project contracts/intelligence-validation --no-sync --offline \
  ruff check --no-cache contracts/intelligence-validation
```

Run the primary project's unchanged formatting, lint and strict typing too.
Use the prepared Node 26.8.1 and Pyright 1.1.411 direct entrypoint described in
the primary guidance, selecting this project's `pyproject.toml` and interpreter
for the separate intelligence check. Require zero diagnostics in both checks.
The intelligence type-check search includes its own source and the primary
source/interface declarations. Both runtime import paths are explicit; the
primary project, configuration and declarations remain unchanged.
Verification never installs or fetches tools.
The isolated project's constraints preserve the accepted primary external
dependency versions, including transitive packages. Compare complete selected
artifact records and actual installed bytes; equal direct pins alone are
insufficient. Dependency changes require their affected qualification.

## Complete combined source qualification

The complete profile selects all thirteen primary modules plus the two registered
modules in `contracts/intelligence-validation/tests/`. Its configuration
adds both public source directories to pytest's import path. Keep primary
`--rootdir` so the thirteen existing modules preserve their node identities.
Both packages have distinct import names; neither shadows the other.

Use the primary guidance's closed environment, real shell/timeout bridge,
exclusive external directories, resource limits, warning assertion and original
45/320-second windows with 30+5/300+5-second timeouts. Set
`PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION=python` in the parent environment. Make
only these explicit selection substitutions in both collection and measurement:

- Select this project's `pyproject.toml` with `-c`; keep the primary project's
  absolute path as `--rootdir`.
- Keep all thirteen primary test-file arguments. Prove this project's test
  directory contains exactly `test_intelligence.py` and `test_records.py`, then append
  that directory exactly once. Directory selection gives the moved module its
  file prefix; selecting an individual file outside `rootdir` loses that prefix
  in the pinned pytest release. Require the resulting 62 framing node identities
  to start with `test_intelligence.py::`, the record identities to start with
  `test_records.py::`, and all 2,667 primary identities and 62 framing identities
  to match their retained ordered collections exactly.
- Select this project's `tests/coverage.ini` for the combined parent measurement.
  It includes both packages and both maintained-helper trees without exclusions.

The actual CLI children continue to use their unchanged primary source paths and
primary `coverage.ini`. Retain and reconcile their physical identities, outcomes,
streams, reaping and raw profiles with the combined parent. Require exact ordered
collection/execution agreement, no required skips, one fresh raw per actual
process and an explicit keep-merge of every compatible raw. Keep assertion-only
rows separate from source/support numerators. Apply strict integer line/branch
gates independently to each executable source/helper, each package, maintained
support and the complete Python repository profile. Prove declaration-only and
zero-branch N/A from source and instrumenter data.

Freeze actual source, fixtures, configuration, complete dependency/artifact closure,
interpreter, imports, callbacks and tools before/after. Repeat on an exact
public-only source copy. The separate combined environment is an intelligence
qualification profile; its results are not a new primary-corpus protocol pass.
Reconcile historical primary acceptance only against its exact unchanged inputs,
tools and selected environment. A changed primary profile still requires its
complete declared 121-command protocol. No gate is removed by this separation.
