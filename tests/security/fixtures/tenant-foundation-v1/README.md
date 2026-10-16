# Tenant foundation qualification inputs

These original public synthetic literals are under MIT OR Apache-2.0. They
extend the independently reviewed [authority fixtures](../authority-v1/README.md)
with canonical-schema and direct-session expectations. They contain no customer
data, operational credentials or actual authentication evidence.

[cases.json](cases.json) pins the complete public source inputs and declares
bounded reads, an explicit synthetic seed, and exact required API, constraint
and migration cases. All 68 case IDs are mandatory. API and direct-constraint
cases start from the complete canonical seed. Migration recipes explicitly start
from their declared version2 or version3 state; no authority seed exists before
version3. Inputs and expected outcomes are separate.
Ordinary checks consume these committed literals without regenerating them.
The consuming reader and recipe implementation are measured owning source.

The full original 25-case tenant group remains mandatory and unchanged. Its RLS
context contains organization and principal only. Neither expected rows nor an
inferred domain may add a prerequisite to those cases. Actual catalogue changes
isolate unsafe ownership, superuser, BYPASSRLS and absent FORCE-RLS. Connection
reuse must retain the same physical backend through the terminal, context-free
middle transaction and second workspace. Query failure is not an empty result.

The additional session cases provide complete organization/domain/principal
contexts explicitly. They test canonical existence/state, retained session state,
read failures, corrupt stored identity and lost terminal completion. Their
controlled faults operate only on the independently prepared isolated database.
The exact constraint cases isolate uniqueness, UUIDv4, profile/key/state bounds
and composite membership/domain references. Migration expectations preserve the
original version1/2 bytes and permit only the declared forward version3 expansion.

Qualification-only probe relations exercise actual RLS and composite typed
relationships. They are not product Project, Party, capture or expense tables.
Their results cannot qualify those product-family implementations or enable any
writer. The application scope guard is separate from current action/delegation
policy. Full current-authority locks/fences/races, shared quotas, bootstrap scope,
authentic continuity, actual authentication, HTTP, pooler, human, recovery and
whole-milestone qualification remain at their consuming gates.

The independent `catalogue_v1` declares exact columns, types, nullability, defaults,
keys, references, check meanings, policy bindings, ownership, RLS and ordinary-role
privileges. Actual catalogue facts and concrete constraint negatives must both
agree. A source hash or matching object count cannot replace this oracle.
`expected_rowsets` supplies complete named canonical afterstates. Every denied
insert preserves the seed; successful inserts have explicit complete afterstates.
All controlled faults restore the seed and catalogue before the next case.

`api_recipes` freezes physical targets, interventions, causal stages, terminal
responses, independent observations and bounded owned cleanup. The revocation
case uses a separately committed update before a second read on the same
READ COMMITTED transaction. The corruption case explicitly names the temporary
invalid UUID and domain-check restoration. The catalogue-failure case temporarily
renames only the selected isolated database's `pg_roles` view, proves SQLSTATE
42P01, then restores its same OID, columns, owner and ACL. It changes no credential.
The terminal-loss cases observe and withhold the actual matching server response;
a timeout alone cannot qualify either result. Error enum names describe this
application boundary and are not registered HTTP problem bodies.

`migration_seed_v2` fixes useful prior record and annotation bytes. The forward,
replay and old-reader rejection recipes preserve those data, original schema and
ordered checksum history. Canonical seeds are inserted only after expansion.
The new fixture's separate read profile is 128 KiB, depth16 and at most128 cases,
allowing complete named afterstates. The original larger authority package keeps
its unchanged admitted read bounds and required25 tenant cases. SQL binding rules
preserve the intended PostgreSQL conversion/constraint boundary rather than
substituting a client-parser rejection.

Twelve additional canonical-table cases read all four real authority tables
through the nonowner role without application WHERE filters. Exact complete
rowsets distinguish policy clauses, both workspaces, missing/partial context,
active nonmembership and separately committed revoked states. Principal and
membership policies intentionally permit selected state inspection; these raw
rowsets must not be replaced with the stricter application admission result.
The qualification-only probe remains separately scoped.

`preparation-cases.json` separately freezes thirty-two reader-preparation cases
within 16 KiB, 32 cases and depth16. They preserve the complete admitted packages
and alter one named discriminator in owned private scratch files. The depth16
positive and depth17 negative retain otherwise valid inputs. Actual file hashes
and sizes are updated in the copied authority descriptor so identity, pin,
completeness and uniqueness failures reach their intended boundary. Source pins
still select the unchanged actual public checkout. Two reference cases exercise
an unsupported family and a missing identity with valid workspace identifiers.
Expected acceptance and errors remain outside the measured preparation helper;
owned scratch cleanup and actual prepared-file hashes are observed separately.
These cases qualify preparation failures only, with no product rejection,
authentication or whole-checkpoint claim. The prior 68 additional and 25 original
tenant expectations are unchanged.

The sixteen appended cases retain the first sixteen case objects exactly and
qualify physical missing, malformed and duplicate-key documents; exact inclusive
additional-file byte limits; authority version and depth limits; unsafe or absent
source pins; and malformed reference identifiers. Byte limits use complete valid
JSON followed by ASCII whitespace. Duplicate keys are actual duplicate members
in the physical JSON, with the complete original document retained. Authority
depth32/33 uses thirty/thirty-one nested arrays beneath an extra root property.
Its copied descriptor binds the actual raw authority bytes, or null SHA/size for
an absent authority file. Each existing selected file receives a real fingerprint
and logical role; absence receives no fabricated hash. Before/after presence and
fingerprints must agree, and all owned files/directories must be removed even
after reader rejection. Safe missing source pins remain absent before/after;
unsafe pins fail before filesystem access. All actual public source pins and
canonical inputs stay unchanged. These cases add no product or service claim.

`error-cases.json` independently fixes the six finite Rust diagnostic labels for
`TenantFailure`. Each case requires the exact displayed label, no underlying
source error, preservation of its typed variant through `Box<dyn Error>` and
propagation of a real formatting-sink failure. Its separate read profile is
4 KiB, eight cases and depth8. The measured observer receives only the selected
variant; expected text and decisions remain in the assertion body. These labels
are diagnostic Rust behavior, not new HTTP problem codes or authentication
evidence. Error interoperability grants no new database or action authority.
