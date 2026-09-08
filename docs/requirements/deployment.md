# Deployment and operation

**Owner:** Engine runtime profiles, packaging and operating procedures.
**Status:** Normative target. No deployment or supported runtime is established by this document.
**Verification:** Actual image, service, recovery and operating-profile evidence under [acceptance](acceptance.md).

A supported deployment is a tested combination of versions, configuration,
resources, dependency behavior and operating procedures. The repository must
provide the public information and committed artifacts needed to operate that
combination. Documentation of a topology is not evidence that it has been
installed, funded or qualified.

[Architecture](architecture.md) owns component responsibilities;
[dependency qualification](dependency-qualification.md) owns exact pins and
targets; [resource profiles](resource-profiles.md) owns capacities and budgets.
Use those requirements together rather than defining different behavior for
local and remotely operated instances.

## Supported-profile progression

| Profile | Required behavior and evidence |
| --- | --- |
| Local development and qualification | Provide explicit setup for pinned API, worker, Python, application PostgreSQL/pooler, object storage and Temporal components. Bind local interfaces safely and start with synthetic data. A local mail sink and retrieval service are enabled when their tests require them. Real inference is an explicit selected route. Measure the H0 profile; a development startup alone is not real-data acceptance. |
| Pilot qualification | Use exact Linux AMD64 image digests, an explicitly protected service network and the selected database, object-storage, identity, inference and mail profiles. Qualify direct and pooled database connections, Temporal's separate persistence and the retrieval generation. Record actual regions, access, resource allocation, costs and failure behavior. |
| Production | Claim only the topology and scale that passed the mandatory gates and experiments. Required recovery, security, budgets, supported client compatibility, signed distributions, incident ownership and support commitments must describe that same profile. |

The supported local profile must not require a paid Arboresce entitlement. It may
use a disclosed, user-selected external inference service under its own terms.
Document preparation separately from startup. Missing credentials, images or
processing resources must produce an actionable unavailable result rather than
triggering an implicit installation or substituting recorded processing.

Single-region operation is permitted when its accepted obligations are met.
Do not advertise high availability solely because the API has multiple replicas.
A single retrieval or storage instance remains an availability boundary for its
dependent capability. Components sharing a machine or volume also share resource
and failure risks; capacity qualification must include that overlap.

## Service and persistence topology

Default local listeners bind loopback. Remote exposure requires explicit identity,
TLS and routing configuration under [security and privacy](security-and-privacy.md).
Expose application interfaces deliberately; database, administration, workflow
and retrieval interfaces are not public merely because the API is reachable.
Use separate service identities and least-privilege roles with explicit access
to the dependencies each role needs.

Application PostgreSQL and Temporal persistence use separate databases, credentials
and connection budgets. Migrations and administration use their declared direct
connection profiles. A transaction pooler must be qualified for the application
session and statement behavior actually used; it is not interchangeable with a
direct connection by assumption.

Persistent components require tested backup and restoration. A volume or a
provider's replication description does not prove application recovery. Recovery
must account for all canonical records, exact object bytes, keys/trust and current
lifecycle restrictions as specified in [recovery](recovery.md).

The worker and Temporal roles must continue the durable work for which they are
responsible without an incoming HTTP request to wake them. Record minimum running
capacity and any scaling rules. Expensive admission and reservations must respect
the shared capacity limits across all allowed instances; adding a replica must
not multiply an organization quota or bypass a cost reservation.

Retrieval and notifications have declared degradation behavior. A retrieval outage
may disable search while exact eligible build resolution continues if its own
dependencies are healthy. An optional mail outage must not invent successful
delivery or reverse a committed domain transition. Canonical policy unavailability
must fail closed for work that requires it.

## Images and packaged resources

Build the API, worker, projector and administration roles from pinned multistage
inputs, with only their required runtime files in the final image. Use non-root
runtime users, restricted writable paths and a read-only filesystem where the
profile has qualified it. Exclude credentials, dependency caches, build tools,
Git metadata, customer material and source unnecessary at runtime from artifacts.
Grant each role only its required privileges.

The Python image must contain its pinned runtime and codec dependencies, required
generated bindings, ordinary prompt resources and selected processing-profile
resources. Startup must not install packages or download unqualified weights.
Apply the parser scratch, inode, CPU, memory and time budgets in the actual image,
including cleanup after failure. A successful parser test outside the image does
not establish the image's resource behavior.

Record the image digest, source and lock identities, target architecture,
dependency inventory, licence notices and configuration contract. Do not treat a
mutable image tag as a release identity. Required generated outputs follow
[generated-artifact guidance](../generated-artifacts.md) and must work during
ordinary use without regeneration.

Native client packaging remains independently qualified and versioned. An engine
release must identify the compatible tested CLI artifacts; distributing an engine
image alone does not establish that users can install and complete the product
journeys.

## Startup, health and shutdown

Validate configuration before accepting work. Reject unknown or incompatible
profiles and missing required resources. Resolve secrets through the configured
protected mechanism; configuration diagnostics must not expose their values.
Provision the first owner using the controlled bootstrap flow rather than a
reusable default credential.

Health behavior follows [the API contract](api.md). A process being alive is
different from that role being ready to serve its required work. Readiness must
reflect required dependency and capacity failures, while declared optional
capabilities report degradation accurately. Responses must not expose service
addresses, SQL, credentials or record content.

On shutdown, stop new admission and lease acquisition, mark readiness accordingly,
then drain bounded accepted work. Propagate cancellation and deadlines through
Activity, RPC, transfer and provider boundaries, release claims appropriately and
terminate within the declared operating budget. Slow clients or external calls
must not keep the process alive indefinitely. Durable work must resume through
the tested retry/recovery mechanism; shutdown must not silently lose an accepted
operation or claim it succeeded.

## Required operating procedures

Publish procedures only when their commands exist and have been exercised in the
selected profile. Each procedure records prerequisites, actor authority, exact
inputs, observable progress, failure handling and evidence of the resulting state.

| Procedure | Required result |
| --- | --- |
| Bootstrap | Validate the environment and storage permissions, install the selected schema through its authorized role, establish the first owner securely and verify health. |
| Deploy and rollback | Identify exact compatible images/contracts, drain safely and use a reviewed expand–backfill–contract sequence. Define the compatibility window and rollback limits; rollback cannot discard accepted data. |
| Temporal upgrade | Verify separate server, tool, schema and SDK pins, the supported upgrade sequence, relevant history replay and old/new worker compatibility. Record any point beyond which rollback is unsupported. |
| Restore | Reconstruct the dependencies and current restrictions required by [recovery](recovery.md); keep serving closed until its reconciliation checks pass. |
| Projection rebuild | Establish the scoped mutation barrier, reconcile a new generation against canonical state, switch safely and retain the permitted rollback generation. |
| Blob reconciliation | Compare staging/sealed objects with canonical references and holds, isolate suspected orphans and perform only authorized bounded cleanup with retained evidence. |
| Credential and key rotation | Exercise replacement and overlap rules for authentication, database, provider, SMTP and storage credentials and signing trust. Verify that recovery cannot revive revoked authority. |
| Incident response | Bound affected access and processing, preserve appropriate evidence, identify the responsible operator, perform required notifications and add a reproducing regression case. |
| Customer export and deletion | Authenticate the scope, produce a completeness result, handle unresolved or held records explicitly and account for object, projection, cache and backup consequences. |
| Overload and cost control | Refuse new expensive work at the declared limits, preserve accepted work, apply fair admission and bounded cancellation, and expose accurate backlog and retry guidance. |

Notification delivery requires a generic application port and a qualified SMTP
implementation. The default submission profile uses certificate-verified TLS on
port `465`; STARTTLS requires an explicitly permitted and tested configuration.
Stable notification identity, template version and Message-ID aid reconciliation
but do not guarantee deduplication. Preserve uncertain acceptance after a broken
connection. Verification uses a controlled mail service instead of real recipient
mailboxes.

## Operational and release evidence

Record real hardware, configuration, versions, region, latency, cold/warm state,
workload, actual counts and costs for the selected profile. Use the telemetry
defined by [resource profiles](resource-profiles.md), with content-minimizing
operational labels. Keep product audit and provider-exposure records distinct
from operational metrics. Define accountable incident, access-recovery, abuse,
patching and support responsibilities without assuming people or services have
already been appointed.

Real-data use and independent-user qualification must respect the packaged
security and restoration prerequisites in [acceptance](acceptance.md). Early
restoration does not replace the later full-scale recovery experiment. A backup
without a successful restore, or a local check without the required independent
verification, is not production evidence.

Prepare distribution manifests, SBOMs, checksums and verification tooling before
release signing. An unsigned candidate remains labelled as such. Actual release
acceptance requires authorized signatures over the distributed artifacts and
independent verification of their digest, length, platform and build identity,
with valid trusted signers. Context-content attestations do not satisfy that
distribution requirement. Follow [release policy](../release-policy.md) and the
complete [acceptance criteria](acceptance.md).
