# Repository guidance

## Purpose

Arboresce helps organizations preserve and develop what they learn.
These instructions apply throughout this repository.

## Scope

Contributors and automated contributors follow this file and the nearest instructions for the files they edit.
Use [the instruction index](docs/agents/README.md) to find directory guidance.
Read the [requirement ownership index](docs/requirements/README.md) before
implementing behavior. Those documents define the accepted target; the
[rolling plan](docs/execution/open-cli-mvp-v1-rcld.md) records execution and evidence.
The current checkout has no product runtime. Use the
[public contract checks](contracts/validation/README.md) for the supplied
schemas and fixtures. Introduce runtime commands with their owning guidance.

## Project voice

Refer to it as `Arboresce`, `the project`, or `this repository`.
Describe the project through its purpose, behavior, and interfaces.
Keep documentation and Git messages grounded in the files and behavior changed here.
Use calm, direct language. Prefer short sentences and concrete terms.

## Working agreement

Keep changes small, clear, and useful on their own. Preserve unrelated work.
Keep credentials and personal information outside the repository.
Use rights-cleared sources and retain their attribution.
Preserve copyright and attribution notices. Add or change a notice only when its owner and year are known.

## Repository integrity

A normal clone must contain everything required for normal builds, tests, documentation, and use apart from documented external dependencies.
Follow [the generated artifact process](docs/generated-artifacts.md) when changing generated files.
Ordinary use must rely on committed artifacts rather than requiring regeneration.

## Verification and completion

Use only documented commands.
Run the documented formatting, checks, and tests for affected components.
Source changes must satisfy [the testing and coverage contract](docs/requirements/testing-and-coverage.md),
including deterministic independently reviewed fixtures, complete collection and
strict coverage for each applicable source, package, repository and profile.
Measure maintained fixture support and all exercised process boundaries. Record
actual results and unresolved qualification inputs; do not treat planned tests
or a successful unit double as service or product qualification.
Review the final diff.
A change that adds or changes a supported command must update its owning guidance in the same change.
Report changed files, commands run, results, skipped checks, and remaining questions.
