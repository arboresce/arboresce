# Arboresce

Arboresce is infrastructure for helping organizations preserve and develop what they learn.

The project is designed to turn evidence, decisions, expert judgment, and outcomes into knowledge that can be reviewed, maintained, and used by people and software.

The accepted first release covers two workflows: evidence-backed expense
recordkeeping with per-currency reports and export, and reviewed knowledge with
versioned context builds, application consumption and feedback. See the
[product requirements](docs/requirements/product-and-scope.md) for their complete
scope and the [requirement owners](docs/requirements/README.md) for the contracts.

This checkout supplies requirements, contribution guidance, licensing and
[checked public contract fixtures](contracts/validation/README.md).
The runtime, libraries, processing service, API and runtime qualification suites
are planned implementation. No product build, installation or runtime command is
available here yet. The [rolling plan](docs/execution/open-cli-mvp-v1-rcld.md)
records implementation and verification separately from accepted requirements.

## Repository

| Directory | Purpose |
| --- | --- |
| `crates/` | Rust libraries and executable packages |
| `contracts/` | Data, API, and extension contracts |
| `docker/` | Reusable image definitions and local runtime examples |
| `docs/` | Requirements, architecture, and contributor documentation |
| `python/` | Python intelligence processing |
| `tests/` | Shared fixtures and cross-component tests |
| `tools/` | Repository tooling and verification |

These directories are ownership boundaries; their presence does not establish
implemented components. The separately maintained CLI uses versioned public
contracts and network interfaces. Normal use must require only this checkout and
documented external dependencies, with no Arboresce entitlement. Users may select
a third-party inference provider with explicit credentials, terms and costs.

Start with [the documentation index](docs/README.md) and [contribution guidance](CONTRIBUTING.md).

## License

Original project material is available under either the [MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE), at your option.
See [NOTICE](NOTICE) for attribution.
