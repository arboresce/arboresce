# Documentation

This index covers the documentation supplied with the repository.

Start with [the MVP requirement owners](requirements/README.md). Their accepted
targets are normative; implementation and measured acceptance remain separate.
The [rolling implementation plan](execution/open-cli-mvp-v1-rcld.md) tracks
each checkpoint and its actual evidence.

The [common wire contract](requirements/common-contract.md),
[acquisition contract](requirements/acquisition-contract.md),
[observation contract](requirements/observation-contract.md),
[candidate review contract](requirements/candidate-review-contract.md),
[financial records contract](requirements/financial-records.md),
[context contract](requirements/context-contract.md) and
[lifecycle and operation contract](requirements/lifecycle-contract.md) define versioned
schema families. Their [standalone checks](../contracts/validation/README.md)
validate independent parsed-value fixtures; runtime qualification follows its
own implementation slices.

Lifecycle uses a [public definition library](../contracts/public/v1/lifecycle.schema.json)
and a separate [internal continuity library](../contracts/internal/v1/lifecycle-continuity.schema.json)
in one checked family. Public contracts remain independent of internal imports;
internal schemas may reuse public definitions. Specified repair/evidence hooks
require complete qualified producers before runtime activation.

- [Instruction index](agents/README.md)
- [Documentation ownership](ownership.md)
- [Release and compatibility policy](release-policy.md)
- [Generated artifacts](generated-artifacts.md)
- [Contribution guidance](../CONTRIBUTING.md)
- [Security reporting](../SECURITY.md)
- [Code of Conduct](../CODE_OF_CONDUCT.md)
