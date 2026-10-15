//! Checked Gregorian dates and exact UTC millisecond values. A supplied clock
//! does not define revision order or a source event's attribution or timezone.

use std::{error::Error, fmt, str::FromStr};

/// A date rejected without retaining its input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidCalendarDate;

impl fmt::Display for InvalidCalendarDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid Gregorian calendar date")
    }
}

impl Error for InvalidCalendarDate {}

/// An instant rejected without retaining its input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidUtcTimestamp;

impl fmt::Display for InvalidUtcTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid UTC millisecond timestamp")
    }
}

impl Error for InvalidUtcTimestamp {}

fn month_length(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

// Every caller supplies at most four bytes, so exact decimal folding fits u16.
fn digits(bytes: &[u8]) -> Option<u16> {
    bytes.iter().try_fold(0, |value, byte| {
        if !byte.is_ascii_digit() {
            return None;
        }
        Some(value * 10 + u16::from(byte - b'0'))
    })
}

/// An immutable Gregorian date in years 0001 through 9999, with no timezone.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CalendarDate {
    year: u16,
    month: u8,
    day: u8,
}

impl CalendarDate {
    /// Check explicit calendar components without constructing an instant.
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, InvalidCalendarDate> {
        if !(1..=9999).contains(&year) || day == 0 || day > month_length(year, month) {
            return Err(InvalidCalendarDate);
        }
        Ok(Self { year, month, day })
    }

    /// Return year, month and day without permitting mutation.
    pub const fn components(self) -> (u16, u8, u8) {
        (self.year, self.month, self.day)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, InvalidCalendarDate> {
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(InvalidCalendarDate);
        }
        let year = digits(&bytes[..4]).ok_or(InvalidCalendarDate)?;
        // Two decimal digits fit u8; malformed bytes have already failed.
        let month = digits(&bytes[5..7]).ok_or(InvalidCalendarDate)? as u8;
        let day = digits(&bytes[8..10]).ok_or(InvalidCalendarDate)? as u8;
        Self::new(year, month, day)
    }

    fn days_from_epoch(self) -> i64 {
        let prior = i64::from(self.year) - 1;
        let prior_years = prior * 365 + prior / 4 - prior / 100 + prior / 400;
        let prior_months: i64 = (1..self.month)
            .map(|month| i64::from(month_length(self.year, month)))
            .sum();
        // 719162 Gregorian days precede 1970-01-01. All supported years fit i64.
        prior_years + prior_months + i64::from(self.day) - 1 - 719162
    }
}

impl FromStr for CalendarDate {
    type Err = InvalidCalendarDate;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(input.as_bytes())
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

/// An immutable exact UTC instant with millisecond precision and no leap second.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UtcTimestamp {
    date: CalendarDate,
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
}

impl UtcTimestamp {
    /// Construct an instant from a checked date and every explicit UTC component.
    pub fn new(
        date: CalendarDate,
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
    ) -> Result<Self, InvalidUtcTimestamp> {
        if hour > 23 || minute > 59 || second > 59 || millisecond > 999 {
            return Err(InvalidUtcTimestamp);
        }
        Ok(Self {
            date,
            hour,
            minute,
            second,
            millisecond,
        })
    }

    /// Return the explicit UTC date, without interpreting a source's local date.
    pub const fn date(self) -> CalendarDate {
        self.date
    }

    /// Return hour, minute, second and millisecond without permitting mutation.
    pub const fn time_components(self) -> (u8, u8, u8, u16) {
        (self.hour, self.minute, self.second, self.millisecond)
    }

    /// Return exact signed milliseconds from 1970-01-01T00:00:00.000Z.
    pub fn unix_milliseconds(self) -> i64 {
        self.date.days_from_epoch() * 86400000
            + i64::from(self.hour) * 3600000
            + i64::from(self.minute) * 60000
            + i64::from(self.second) * 1000
            + i64::from(self.millisecond)
    }
}

impl FromStr for UtcTimestamp {
    type Err = InvalidUtcTimestamp;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let bytes = input.as_bytes();
        if bytes.len() != 24
            || bytes[10] != b'T'
            || bytes[13] != b':'
            || bytes[16] != b':'
            || bytes[19] != b'.'
            || bytes[23] != b'Z'
        {
            return Err(InvalidUtcTimestamp);
        }
        let date = CalendarDate::from_bytes(&bytes[..10]).map_err(|_| InvalidUtcTimestamp)?;
        // Checked two-digit fields fit u8. Byte slices never index UTF-8 strings.
        let hour = digits(&bytes[11..13]).ok_or(InvalidUtcTimestamp)? as u8;
        let minute = digits(&bytes[14..16]).ok_or(InvalidUtcTimestamp)? as u8;
        let second = digits(&bytes[17..19]).ok_or(InvalidUtcTimestamp)? as u8;
        let millisecond = digits(&bytes[20..23]).ok_or(InvalidUtcTimestamp)?;
        Self::new(date, hour, minute, second, millisecond)
    }
}

impl fmt::Display for UtcTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}T{:02}:{:02}:{:02}.{:03}Z",
            self.date, self.hour, self.minute, self.second, self.millisecond
        )
    }
}

/// A fallible supplied UTC clock. There is no default or ambient implementation.
/// Repeated or backward instants remain valid and imply no revision ordering.
pub trait Clock {
    type Error;

    fn now(&mut self) -> Result<UtcTimestamp, Self::Error>;
}
