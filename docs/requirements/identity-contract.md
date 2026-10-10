# Nostr identity and scoped execution

**Authority:** normative first identity profile and project execution authority.
**Status:** accepted target; cryptography, key storage, HTTP authentication and
actual-human assurance are not qualified by this document.

[Security](security-and-privacy.md) owns current authorization and exposure controls;
[financial records](financial-records.md) owns exact review-event consumption;
[API](api.md) owns admission/idempotency. This first profile replaces a mandatory
OIDC issuer/browser login prerequisite. OIDC may be introduced later only through
its own qualified profile; no email, signup or paid Arboresce entitlement is needed.

## Principal and bootstrap

A user can create or import a Nostr keypair. Its public key identifies key control
using the qualified NIP-01 secp256k1/BIP-340 signature profile, not a person's name,
email, physical presence, legal identity or approval assurance. The canonical
principal mapping includes the identity-profile/version and exact public key.
Nostr relay operation or discovery is not required for engine HTTP identity.
Parties and aliases are separate domain records; matching names cannot claim keys.
Content/build/artifact Ed25519 signing and cursor HMAC keys remain separate trust
domains. Never substitute a Nostr signature for their existing exact contracts.

Keypair creation is local identity preparation. Server registration/authentication
proves control through the qualified signed-request flow. A first workspace requires
the documented operator-held, expiring, one-use bootstrap grant; there are no
reusable default credentials or unauthenticated tenancy exceptions. Define and
qualify exact registered bootstrap command, namespaces, policy and attribution
before admission. Authentic independently retained recognition/lifecycle continuity
and its fail-closed interpreter precede the first consuming writer, including the
first owner. Ordinary commands then require explicit one-workspace membership,
domain/purpose policy and action authority.

## Signed HTTP profile

Pin the exact revisions of [NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md)
and [NIP-98](https://github.com/nostr-protocol/nips/blob/master/98.md), libraries,
algorithm/encoding rules and independent fixed vectors before first consumer.
Use the signed NIP-98 HTTP authorization event with exact method, externally
addressed absolute URL including query, empty content, allowed event kind and
required created-time validation. Arboresce requires SHA-256 binding of every
nonempty exact HTTP body; NIP-98's optional body verification is insufficient.
Define and test bodyless commands, uploads, streams and server-derived origin
behind any trusted proxy. Never reconstruct signed URLs from arbitrary forwarding
headers or verify a parsed/re-serialized body instead of the admitted bytes.

The profile must freeze finite clock tolerance, authoritative clock-health rules,
event size/tag cardinality, allowed schemes/origins, replay retention and all
parser/header limits. Reject duplicate/contradictory tags, unsupported encodings,
invalid signatures, changed method/URL/body, expired/future/replayed events and
unknown or revoked principal authority. Profiles without these selections cannot
admit work. TLS and destination policy remain required independently of signatures.

Replay protection and command idempotency are distinct. A signed event cannot
authorize a changed request or fresh action. A transport retry signs a fresh valid
request while retaining the original command idempotency key and intent. Under
current reauthorization, identical admitted intent returns its original result;
different intent with that key conflicts. Define and test lost response, concurrent
duplicate, body mutation, clock rollback, revocation and replay-expiry boundaries.
Retain bounded authentic replay evidence under selected retention policy; absence
or expiry must not turn a recognized durable command into a new intent.

## Key lifecycle and agent grants

Use qualified operating-system secret storage or an explicitly qualified protected
file profile. Never expose secret keys in arguments, logs, JSON output, diagnostics,
examples or fixture exports. Provide intentional create/import, backup/restore,
local logout, server revocation and replacement procedures. Logout immediately
fences the selected local profile while preserving unrelated profiles, exports and
unsent edits; a concurrent signer cannot restore logged-out use. Server revocation
is separate and remains observable when remote cleanup fails. Key loss cannot be
recovered from an email assumption. Key replacement needs a qualified current
recovery authority and explicit same-principal transition or a new principal with
reviewed membership migration; never infer continuity from an alias.

Agents have their own keys/principals and explicit grants. Do not lend an agent the
owner's key and rely on prompt constraints. Grants bind workspace/domain, project
or resource scope, permitted actions/purposes, delegability, lifetime, policy and
revision. Validate each immutable chain and its current restrictions at every
admission and final effect fence. Delegation cannot increase the issuer's authority
or human assurance. Record actual executor and exact selected chain separately
from attributed source author or human reviewer.

An explicit project-update grant may permit an agent to review and accept typed
project items without a fresh human prompt. It cannot grant financial human-review
assurance or expand to arbitrary tools, permissions, cross-workspace writes or
consequential external actions. Existing human-required knowledge/build policies
retain their stronger requirements. Revocation denies subsequent use and affected
replay disclosure, without rewriting historical attributable decisions.

## Trusted financial review-event issuance

Financial confirmation requires a distinct actual-human decision, not mere key
possession, a CLI confirmation flag, a model statement or a general grant. The
configured financial review policy must name a concrete qualified human-assurance
and trusted signer/review mechanism before event issuance or confirmation support.
That mechanism is an explicit unqualified input until selected and proved; fixtures
and a server label cannot qualify actual human presence or approval.

The issuance flow must be implemented as follows:

1. An authenticated requester selects the exact pending expense revision/digest,
   optional exact project association, financial purpose and permitted executor.
   The server returns a bounded, expiring one-use review challenge and a readable
   exact-content rendering plus its independently verifiable identity.
2. The qualified review mechanism authenticates the actual human, presents that
   exact consequential content and collects the explicit `confirm` decision.
   It binds challenge, expense and association identities, policy and executor
   scope. An agent holding its own key cannot produce this assurance.
3. The server verifies the mechanism's proof and current reviewer authority,
   challenge freshness/single-use and exact content under the selected policy.
   It durably issues `financial_review_event_id` with reviewer, assurance,
   workspace/domain/purpose, content identity, expiry, revocation and executor
   scope. Issuance and challenge consumption are atomic and attributable.
4. The human or separately scoped agent submits the financial confirmation with
   that event. The financial owner rechecks and consumes it exactly once in the
   canonical confirmation transaction. Idempotent replay retains the committed
   result under current disclosure authorization.

Before consumers, freeze complete challenge/proof/issue/result contracts, registered
routes/actions and canonical preimages; no route is enabled by this document.
Prove stale revision/association, tampering, fake assurance, revoked reviewer or
executor, expiry, competing issuance/confirmation, lost responses and restart.
Financial review changes neither candidate verdicts nor project update approval.

### Compound composition assurance

The separate `financial_composition_review.*` operations challenge and issue one
actual-human decision over the exact composition, project, expense and association
and their current controls. Their versioned preimage, challenge, rendering and
proof bind the selected review policy, assurance, human reviewer, permitted executor,
exclusive lifetime and exact content. The proof has its own compound kind; it is
never presented as a legacy expense-only proof. A claimed `qualified` fixture label
does not qualify a real mechanism, signer, human presence or consent.

The issued compound wrapper stores the exact canonical legacy
`financial_review_event` base portion at issuance plus the composition identity and
compound preimage/digest. It contains one event ID and one human decision, governed
by one global single-use consumption record. Issuance consumes the challenge
atomically; confirmation consumes the event atomically with all compound effects.
The compound event ID, challenge, reviewer, assurance, policy, executor, purpose,
workspace, expense, association and lifetime must agree with its base portion.
The compound digest also binds independent project/expense/association/composition
controls. Rendering hashes exact UTF-8 text and identifies that compound digest.

The compound rendering kind fixes the deterministic text profile. Its UTF-8 text
is exactly `Arboresce financial composition review v1` followed by LF and the
bounded canonical JSON serialization of the closed display object with four
members: `composition` (complete immutable composition content), `association`
(complete exact association), `expense_content` (unchanged closed financial
content) and `review_preimage` (complete compound preimage, including independent
controls). There is no trailing newline. The rendering digest hashes these exact
bytes; `review_sha256` hashes the separate complete compound preimage. This
renders every monetary fact and exact target instead of substituting a summary.
The configured actual-human mechanism must present this complete consequential
content and verify its identity; deterministic fixture rendering does not qualify
that mechanism or a human decision. A different rendering profile requires its
own explicit version and qualification.

Legacy event resolution may return this exact stored base portion under current
authorization, without reconstructing or manufacturing it on the client. Legacy
challenge issuance and expense-only confirmation reject compound challenge/event
authority. A compound ID cannot be downgraded into a partial expense-only commit,
or consumed twice through two routes. Old expense-only authority cannot confirm a
composition. Complete current mechanism qualification, reviewer/executor authority,
delegation, revocation and time fences remain mandatory runtime inputs.

## Self-principal server revocation

The API transport-completeness contract defines authenticated self-principal
revocation with exact current control revision and explicit reason. It revokes
that server's principal/key authority and returns the qualified revoked identity;
remote deletion and recovery are separate. It does not request key transition,
select a replacement principal or bypass recovery authority. Current authorization,
final effect fencing and retained idempotency receipt rules still apply; revocation
cannot grant a revoked signer renewed authority for a retry or unrelated read.

A successful revocation response contains a principal in `revoked` state with
a non-null canonical `revoked_at` timestamp. Human, agent and service principals
retain their original kind and complete key-control profile. An active identity
is not a successful revocation result, even when it otherwise satisfies the
principal resource schema.

Parsed-value checks enforce that state and timestamp shape. The consuming service
and client must also compare the returned principal ID with the authenticated
self-principal and exact admitted request, compare the inner principal control
revision with the result control observation, and verify the expected-to-result
control transition against the canonical transaction. JSON Schema does not
establish these cross-record equalities. They require independently expected
semantic cases before consumer qualification.
