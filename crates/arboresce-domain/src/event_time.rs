//! Attributed source/user time and explicit UTC calendar projections. Checked
//! references and supplied clock values establish no provenance or authority.

use crate::{
    CalendarDate, Clock, ContentRevision, Identifier, OriginalReference, RepresentationReference,
    UtcTimestamp,
};
use std::{error::Error, fmt};

/// Rejected observed precision or text, without retaining the input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidObservedTime;

impl fmt::Display for InvalidObservedTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid observed time")
    }
}

impl Error for InvalidObservedTime {}

/// A required explicit calendar choice that could not be applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimePolicyError {
    MissingPolicy,
    UnsupportedProfile,
    ForeignOrganization,
}

impl fmt::Display for TimePolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingPolicy => "missing explicit UTC calendar policy",
            Self::UnsupportedProfile => "unsupported UTC calendar profile",
            Self::ForeignOrganization => "calendar policy organization mismatch",
        })
    }
}

impl Error for TimePolicyError {}

/// A distinct recorded UTC value. Typed construction does not read a real clock.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RecordedTime(UtcTimestamp);

impl RecordedTime {
    pub const fn new(timestamp: UtcTimestamp) -> Self {
        Self(timestamp)
    }

    pub const fn timestamp(self) -> UtcTimestamp {
        self.0
    }
}

impl fmt::Display for RecordedTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Admitted source precision. A date has no implied instant or source timezone.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ObservedValue {
    Date(CalendarDate),
    Instant(UtcTimestamp),
}

impl ObservedValue {
    /// Pure text intake, separate from every owning operation's JSON wire shape.
    /// Unsupported precision or local/offset text never becomes an absent value.
    pub fn from_text(precision: &str, input: &str) -> Result<Self, InvalidObservedTime> {
        match precision {
            "date" => input
                .parse()
                .map(Self::Date)
                .map_err(|_| InvalidObservedTime),
            "instant" => input
                .parse()
                .map(Self::Instant)
                .map_err(|_| InvalidObservedTime),
            _ => Err(InvalidObservedTime),
        }
    }
}

/// Immutable asserted identity. These references confer no authorship or access.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TimeAttribution {
    Original(OriginalReference),
    Representation(RepresentationReference),
    Principal {
        organization_id: Identifier,
        principal_id: Identifier,
    },
}

impl TimeAttribution {
    pub const fn organization_id(self) -> Identifier {
        match self {
            Self::Original(original) => original.organization_id(),
            Self::Representation(representation) => representation.original().organization_id(),
            Self::Principal {
                organization_id, ..
            } => organization_id,
        }
    }
}

/// A source/user temporal assertion retaining both precision and attribution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObservedTime {
    value: ObservedValue,
    attribution: TimeAttribution,
}

impl ObservedTime {
    pub const fn new(value: ObservedValue, attribution: TimeAttribution) -> Self {
        Self { value, attribution }
    }

    pub const fn value(self) -> ObservedValue {
        self.value
    }

    pub const fn attribution(self) -> TimeAttribution {
        self.attribution
    }

    pub const fn organization_id(self) -> Identifier {
        self.attribution.organization_id()
    }

    /// Project under an explicitly supplied matching foundation policy. This
    /// preserves a date's precision and does not interpret an unspecified zone.
    pub fn reporting_date(
        self,
        policy: Option<UtcCalendarPolicy>,
    ) -> Result<ReportingDate, TimePolicyError> {
        let policy = policy.ok_or(TimePolicyError::MissingPolicy)?;
        if policy.organization_id != self.organization_id() {
            return Err(TimePolicyError::ForeignOrganization);
        }
        let date = match self.value {
            ObservedValue::Date(date) => date,
            ObservedValue::Instant(instant) => instant.date(),
        };
        Ok(ReportingDate {
            date,
            observed: self,
            policy,
        })
    }
}

/// Immutable receipt/event pairing. No chronology or revision order is imposed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EventTimes {
    recorded: RecordedTime,
    observed: Option<ObservedTime>,
}

impl EventTimes {
    pub const fn new(recorded: RecordedTime, observed: Option<ObservedTime>) -> Self {
        Self { recorded, observed }
    }

    pub const fn recorded(self) -> RecordedTime {
        self.recorded
    }

    pub const fn observed(self) -> Option<ObservedTime> {
        self.observed
    }

    /// Read the supplied clock exactly once and preserve its exact caller error.
    pub fn record<C: Clock + ?Sized>(
        clock: &mut C,
        observed: Option<ObservedTime>,
    ) -> Result<Self, C::Error> {
        clock
            .now()
            .map(|timestamp| Self::new(RecordedTime::new(timestamp), observed))
    }
}

/// Explicit versioned UTC foundation choice, separate from actual organization
/// policy membership, IANA rules and later period/deadline consumers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UtcCalendarPolicy {
    organization_id: Identifier,
    policy_id: Identifier,
    version: ContentRevision,
}

impl UtcCalendarPolicy {
    pub fn new(
        organization_id: Identifier,
        policy_id: Identifier,
        version: ContentRevision,
        profile: &str,
    ) -> Result<Self, TimePolicyError> {
        if profile != "utc_calendar_v1" {
            return Err(TimePolicyError::UnsupportedProfile);
        }
        Ok(Self {
            organization_id,
            policy_id,
            version,
        })
    }

    pub const fn organization_id(self) -> Identifier {
        self.organization_id
    }

    pub const fn policy_id(self) -> Identifier {
        self.policy_id
    }

    pub const fn version(self) -> ContentRevision {
        self.version
    }

    pub const fn profile(self) -> &'static str {
        "utc_calendar_v1"
    }

    pub const fn timezone(self) -> &'static str {
        "UTC"
    }
}

/// Complete immutable projection, retaining the original assertion and policy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReportingDate {
    date: CalendarDate,
    observed: ObservedTime,
    policy: UtcCalendarPolicy,
}

impl ReportingDate {
    pub const fn date(self) -> CalendarDate {
        self.date
    }

    pub const fn observed(self) -> ObservedTime {
        self.observed
    }

    pub const fn policy(self) -> UtcCalendarPolicy {
        self.policy
    }
}
