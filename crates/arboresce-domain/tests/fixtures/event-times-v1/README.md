# Independent event-time fixtures

These original public synthetic literals use MIT OR Apache-2.0. They extend the
existing date, exact UTC millisecond and injected-clock vectors. Their owning
requirements are the data model's distinct recorded/observed meanings and the
common calendar/timestamp primitives. The pinned later project, candidate-review,
financial and feedback contracts retain their own wire shapes and actual gates.
This package specifies pure typed foundation behavior; it defines no JSON wire
union, public route, actual clock, policy membership or authenticated attribution.

RecordedTime retains an exact UtcTimestamp. EventTimes pairs it with an optional
ObservedTime, whose value is a CalendarDate or UtcTimestamp and whose attribution
retains the complete original reference, representation reference, or asserted
organization/principal identifiers. None means absent observed time. It is never
the successful fallback for a parse, policy or clock error. ObservedValue's pure
from_text factory accepts exact precision names date/instant and delegates to the
unchanged checked primitives. All other names or invalid texts return the static
InvalidObservedTime error, displayed as `invalid observed time`.

Date-only values never gain midnight, an instant or an inferred source timezone.
Observed time can precede or follow receipt time; neither orders revisions.
Direct typed RecordedTime/EventTimes construction is value pairing, not an actual
clock observation. EventTimes::record invokes the supplied Clock exactly once,
including for absent observed time, and propagates its exact caller-owned error.
Successful prior values survive later repetitions, backwards times and failures.
Two correction pairs retain old and new observed, attribution and receipt meanings.
No period, deadline, persisted revision history or serving transition is introduced.

UtcCalendarPolicy requires explicit organization, policy ID, content revision and
the exact profile name `utc_calendar_v1`. Its declared calendar timezone is UTC.
No normalization, default, OS timezone, tzdb or IANA resolver is consulted.
ObservedTime::reporting_date requires a supplied policy for the attribution's exact
owning organization: original.organization_id, representation.original.organization_id,
or principal.organization_id. It retains the complete observed value/attribution,
full policy reference/profile/timezone and resulting date. A date preserves itself;
an instant uses its explicit UTC date. This does not retrospectively give a source
date an instant or timezone. Missing policy, unsupported profile and foreign
organization have distinct static TimePolicyError variants/messages frozen here.

The five groups contain 31 complete pairs/corrections, 42 explicit policy outcomes,
12 exact clock sequences, 37 precision/text cases and 32 complete identity pairs.
Expected getter projections, exact clock errors/call counts, UTC boundaries and
same-date identity differences are literal data admitted before implementation.
Clone/copy equality, HashSet cardinality, formatter failure and input-independent
error expectations are explicit. Public support is measured with production code.

The three attribution variants preserve asserted identity only. Checked IDs and
references prove no actual existence, original authorship, truth or access.
The explicit UTC foundation is not an organization's default reporting policy.
Later project relative dates, candidate periods/review dates and financial reports
still require their selected IANA timezone/rules, policy revision, actual membership,
authority and DST/local ambiguity qualification. Actual clocks, original source
truth, persistence, human/model/recovery/native/user/production gates remain required.
Source JSON/JCS, aggregate/selected budgets, actual JSON Pointer selection and
whole domain-checkpoint acceptance are separate remaining requirements.
