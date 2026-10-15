# Independent date, timestamp and clock value fixtures

These are original public synthetic fixtures under MIT OR Apache-2.0. They
contain no real records, credentials, private originals or customer provenance.
The [common contract](../../../../../docs/requirements/common-contract.md) and
[data model](../../../../../docs/requirements/data-model.md) own the behavior.

[cases.json](cases.json) pins those inputs and the unchanged
[primitive semantics](../primitives-v1/semantics.json). Its three original groups
retain all 21 dates, 18 timestamps and the supplied repeated/backward clock case.
Additional literal construction, lexical, trait and formatter cases precede the
implementation. UTC integer-millisecond expectations were independently checked
with Gregorian calendar arithmetic, including the pre-epoch value minus one.
These data are not executable oracle helpers or a second normative specification.

`CalendarDate` stays a date with no implied instant or timezone. `UtcTimestamp`
accepts exactly UTC with three fractional digits, years 0001 through 9999, and
valid clock components; it rejects leap seconds, local ambiguity, alternate
offsets and precision. An explicit constructor requires the date and every time
component. A fallible injected `Clock` supplies checked timestamps; reads preserve
the supplied order, repeats, backward movement and caller-defined failures. No
default ambient implementation, revision ordering or user-observed attribution
is supplied. Whole wire/JCS/budget/locator and actual runtime gates remain required.
