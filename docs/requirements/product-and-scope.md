# Product and scope

**Owner:** Engine product requirements.
**Status:** Normative target. Product implementation and qualification remain pending.
**Verification:** The journeys and public capability checks in [acceptance](acceptance.md), using the [testing contract](testing-and-coverage.md).

Arboresce lets people preserve evidence, examine proposed interpretations, maintain
reviewed knowledge and use exact versions of that knowledge in software. It must
retain qualifications, disagreement and unsuccessful outcomes as well as accepted
conclusions. An accepted claim has an accountable review history and a stated
scope; acceptance does not make it universally true.

These requirements describe the intended product. At their introduction, this
repository contains documentation, instructions and licensing, with no product
manifest or implemented runtime. The [implementation plan](../execution/open-cli-mvp-v1-rcld.md)
records delivery and verification separately. An interface described here is not
an available command until its implementation and owning checks pass.

## Users and control

The initial product serves a technically capable individual or small team that
wants to retain and use its own evidence. A personal installation uses the same
organization model as a team, with one member. It does not create a second set of
identity, authorization or financial rules.

The initial product complements users' existing industry-specific applications;
it does not replace them or require one customer's industry model or integration.
Its reusable evidence and review model must remain useful across those contexts.

People must be able to inspect the material behind a proposal, correct it, decide
what to accept, limit its use and obtain a usable export. Review must remain
readable to a person; displaying only an unstructured machine response does not
satisfy the review requirement. Software clients may prepare proposals and submit
authorized commands, but a client option cannot establish human identity or grant
approval authority.

The product distinguishes original evidence, statements supplied by a user,
machine-derived observations, proposed interpretations and reviewed results.
It must not silently turn one category into another. A matching byte digest
establishes content identity, not authenticity. A statement about a meeting does
not establish another participant's agreement, identity or consent.

## Recordkeeping journey

A user can submit a receipt together with a written or spoken note, inspect the
processing result and correct the proposed record. The system preserves the
original evidence and identifies which fields were extracted, supplied by the
user or inferred. Business purpose, participants and reported outcomes are
optional assertions with their own access restrictions.

The user can resolve required financial fields, confirm an exact record revision,
select a reporting period and produce a report grouped by currency. Corrections,
refunds and voids preserve prior history. Reports and exports identify their
selected revisions and applicable calculation rules. An authorized explanation
of excluded or unresolved records accompanies the result without revealing
inaccessible records.

This journey must be useful without creating or publishing a knowledge asset.
The [financial record requirements](financial-records.md) own confirmation,
arithmetic, reporting and export rules. The product does not infer tax treatment
from a receipt, location or model response.

## Reviewed knowledge journey

A user can associate evidence with a proposed claim, state its applicability and
assumptions, attach supporting and opposing material, and request review. The
reviewer can inspect the exact candidate and evidence versions, revise the
proposal, reject or contest it, or accept an eligible revision. Revision and
acceptance are distinct actions.

Acceptance produces an immutable asset version with attributable review. A user
can select eligible versions, compile an exact Context Build, evaluate that build
and promote it through a controlled channel. An application or agent can resolve
the exact eligible build, receive its usage receipt and identify the versions it
used. Current access and lifecycle restrictions still apply to previously built
content. A signature does not override those restrictions or certify factual
correctness.

Feedback and outcomes refer to the exact content used. Preserve reported outcomes
separately from independently observed results, including negative results,
alternative explanations and disagreement. Later evidence can narrow, contest or
supersede a lesson through review; it cannot rewrite an earlier result silently.
These version-linked records are required without introducing a general-purpose
experiment or decision-management application.

The first consumer example may demonstrate exact resolution before feedback
support exists. The complete product journey requires the later feedback and
reviewed-revision extension. [API requirements](api.md) own operation semantics;
[the data model](data-model.md) owns identities, relationships and state.

## Public interfaces and independent operation

The engine exposes a documented public API for the independently distributed CLI
and at least one external application or agent. The CLI is a client of that API;
it cannot rely on direct database access or an alternate implementation of
canonical calculations. Operator-only operations must be identified explicitly
in the public capability description.

Every substantive operation supported by a first-party application must have a
supported public API and CLI equivalent. Given the same input, configuration and
authorization, clients receive the same canonical rules and deterministic
results. Differences in device interaction or presentation do not justify
different approval, access, reporting or export semantics. Probabilistic
processing may vary in wording while retaining the same validation, policy and
review requirements.

A normal clone must provide the committed resources needed for ordinary builds,
tests, documentation, packaging and use, together with documented external
dependencies. It must not require another source checkout, an undisclosed
essential endpoint or regeneration with unavailable tooling. Original project
material remains available under `MIT OR Apache-2.0`, with existing notices
preserved.

Core use requires no paid Arboresce account or entitlement. A user may explicitly
select a third-party inference service that requires credentials or payment.
The supported processing profile must disclose its terms, data exposure, costs
and limits. At least one actual processing route must qualify; prerecorded
responses do not provide that route. A credential-free local model is not a
required MVP profile. Loss of an optional paid integration must not remove access
to otherwise available stored records or exports.

## Included and deferred work

The MVP includes secure setup and scoped identity; organization and domain access;
bounded local and controlled HTTPS capture; supported text, image and short-audio
processing; grounded proposals; readable revision-safe review; confirmed financial
records and per-currency exports; asset history; exact builds and channels;
version-linked feedback; bounded retrieval; revocation, holds, purge and complete
customer export. Required recovery and abuse/resource controls accompany those
capabilities. [Processing](processing.md), [security](security-and-privacy.md) and
[resource profiles](resource-profiles.md) define their supported boundaries.

The release must include a documented self-operated engine profile, operator
bootstrap and diagnostics, and compatibility with independently built native CLI
artifacts. Required native client platforms and service-image targets are owned
by [dependency qualification](dependency-qualification.md). Supported operation
and distribution requirements are owned by [deployment](deployment.md).

The following require a later explicit requirement and qualification decision:

- Broader PDF or video support, authenticated source connectors, redirects,
  additional native platforms, and additional agent integration protocols.
- Graph-database features, tracked context, multi-region writes, general model
  training or fine-tuning, stable embedded language bindings, and broad
  application or integration suites.
- Extensive portfolio, experimentation or decision-management interfaces beyond
  the evidence and version-linked outcome records required above.

Tax filing, payments, payroll, autonomous financial decisions, healthcare or
child-specific applications, employee surveillance and productivity scoring are
outside this release. Their absence must not remove the recordkeeping, privacy,
correction or export requirements of the included product.

Production claims additionally require the actual security, capacity, live-quality,
recovery, signed-distribution and independent-user evidence in
[acceptance](acceptance.md). Documentation completeness, a successful submission
or a plausible model result alone does not establish product acceptance.
