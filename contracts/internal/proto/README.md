# Intelligence descriptor generation

The source of truth is `arboresce/intelligence/v1/processing.proto`. The
generation contract `intelligence-generation-v1.json` pins the complete source,
compiler image, its libraries, pure Python protobuf runtime and descriptor.
Ordinary contract consumption reads the committed descriptor and requires no
compiler or automatic download.

The descriptor-only recipe runs from the repository root. Select an existing
qualified protoc 36.2 matching every declared image/library hash, and an empty,
fresh owned `GENERATION_ROOT` outside the source tree. On a routed workstation,
run the router's diagnostic first and route the actual command:

```sh
cargo extbuild doctor
cargo extbuild run -- protoc \
  --proto_path=contracts/internal/proto \
  --include_imports \
  --descriptor_set_out="${GENERATION_ROOT}/processing.pb" \
  arboresce/intelligence/v1/processing.proto
```

The only promised output of this recipe is `processing.pb`, committed at
`arboresce/intelligence/v1/processing.pb`. No imports or plugins are selected.
Repeat in a second independent fresh directory, require both actual commands
to succeed, and compare complete descriptor bytes with the committed output.
Retain exact source/tool/runtime inputs and command outcomes. Do not hand-edit
the descriptor or reuse a successful file after a failed generation command.

Dynamic descriptor decoding is a conformance consumer. Rust and Python service
binding outputs, their complete pinned generators/plugins and actual gRPC
integration remain separate requirements of the first generated-service
consumer. This descriptor contract neither promises those outputs nor claims
that requirement complete. Native resource, service, authorization and recovery
qualification remain at their actual consumers.
