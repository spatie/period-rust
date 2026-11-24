use crate::{Error, IntoDate};
use jiff::{Span, ToSpan, Unit, Zoned, civil::DateTime, tz::OffsetConflict};
use std::{fmt, str::FromStr};

/// The smallest unit used to round, iterate, and compare dates.
///
/// Ordering goes from coarsest ([`Year`](Self::Year)) to finest ([`Second`](Self::Second)).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Precision {
    /// Calendar years.
    Year,
    /// Calendar months.
    Month,
    /// Calendar days (the default).
    #[default]
    Day,
    /// Hours.
    Hour,
    /// Minutes.
    Minute,
    /// Seconds.
    Second,
}

impl Precision {
    /// Every precision from year to second.
    pub const ALL: [Self; 6] = [
        Self::Year,
        Self::Month,
        Self::Day,
        Self::Hour,
        Self::Minute,
        Self::Second,
    ];
    /// A single unit of this precision.
    pub fn interval(self) -> Span {
        match self {
            Self::Year => 1.year(),
            Self::Month => 1.month(),
            Self::Day => 1.day(),
            Self::Hour => 1.hour(),
            Self::Minute => 1.minute(),
            Self::Second => 1.second(),
        }
    }
    /// The PHP interval field name (`y`, `m`, `d`, `h`, `i`, or `s`).
    pub const fn interval_name(self) -> &'static str {
        match self {
            Self::Year => "y",
            Self::Month => "m",
            Self::Day => "d",
            Self::Hour => "h",
            Self::Minute => "i",
            Self::Second => "s",
        }
    }
    /// A strftime format with exactly this precision.
    pub const fn date_format(self) -> &'static str {
        match self {
            Self::Year => "%Y",
            Self::Month => "%Y-%m",
            Self::Day => "%Y-%m-%d",
            Self::Hour => "%Y-%m-%d %H",
            Self::Minute => "%Y-%m-%d %H:%M",
            Self::Second => "%Y-%m-%d %H:%M:%S",
        }
    }
    /// Whether this precision is finer than another.
    pub fn higher_than(self, other: Self) -> bool {
        self > other
    }
    /// Rounds down without losing the occurrence of a repeated hour.
    ///
    /// # Errors
    /// Returns an error for an invalid date or an out-of-range result.
    pub fn round_date(self, date: &Zoned) -> Result<Zoned, Error> {
        let dt = date.datetime();
        let rounded = DateTime::new(
            dt.year(),
            if self >= Self::Month { dt.month() } else { 1 },
            if self >= Self::Day { dt.day() } else { 1 },
            if self >= Self::Hour { dt.hour() } else { 0 },
            if self >= Self::Minute { dt.minute() } else { 0 },
            if self >= Self::Second { dt.second() } else { 0 },
            0,
        )?;
        resolve_fields(date, rounded, self >= Self::Hour)
    }
    /// Returns the last second within the date's precision unit.
    ///
    /// # Errors
    /// Returns an error for an out-of-range result.
    pub fn ceil_date(self, date: &Zoned) -> Result<Zoned, Error> {
        let month = if self >= Self::Month {
            date.month()
        } else {
            12
        };
        let month_start = jiff::civil::Date::new(date.year(), month, 1)?;
        let dt = DateTime::new(
            date.year(),
            month,
            if self >= Self::Day {
                date.day()
            } else {
                month_start.days_in_month()
            },
            if self >= Self::Hour { date.hour() } else { 23 },
            if self >= Self::Minute {
                date.minute()
            } else {
                59
            },
            if self >= Self::Second {
                date.second()
            } else {
                59
            },
            0,
        )?;
        resolve_fields(date, dt, self >= Self::Hour)
    }
    /// Adds one unit, then rounds down.
    ///
    /// # Errors
    /// Returns an error if the date exceeds the supported range.
    pub fn increment(self, date: &Zoned) -> Result<Zoned, Error> {
        self.round_date(&self.shift(date, 1)?)
    }
    /// Subtracts one unit, then rounds down.
    ///
    /// # Errors
    /// Returns an error if the date exceeds the supported range.
    pub fn decrement(self, date: &Zoned) -> Result<Zoned, Error> {
        self.round_date(&self.shift(date, -1)?)
    }
    /// Calendar difference between two dates, normalized to years through seconds.
    ///
    /// Missing midnights do not shorten whole calendar-day differences.
    ///
    /// # Errors
    /// Returns an error if date arithmetic exceeds the supported range.
    pub fn diff(self, start: &Zoned, end: &Zoned, absolute: bool) -> Result<Span, Error> {
        let (one, two) = if start.datetime() <= end.datetime() {
            (start, end)
        } else {
            (end, start)
        };
        if one.time_zone() == two.time_zone() && two < one {
            // timelib normalizes a reversed clock reading using the minute/second
            // fields and offset change, even when the fold spans multiple hours.
            let offset_change = two.offset().seconds() - one.offset().seconds();
            let seconds = (i64::from(two.minute() - one.minute()) * 60
                + i64::from(two.second() - one.second())
                - i64::from(offset_change))
            .abs();
            let nanos = seconds * 1_000_000_000
                + i64::from(two.subsec_nanosecond() - one.subsec_nanosecond());
            let span = Span::new()
                .nanoseconds(nanos)
                .round(jiff::SpanRound::new().largest(Unit::Hour))?;
            return Ok(if absolute || end >= start {
                span
            } else {
                -span
            });
        }
        let span = if start.time_zone() != end.time_zone() {
            let adjusted = end.datetime().checked_add(
                i64::from(start.offset().seconds() - end.offset().seconds()).seconds(),
            )?;
            start.datetime().until((Unit::Year, adjusted))?
        } else if self <= Self::Day
            && *start == self.round_date(start)?
            && *end == self.round_date(end)?
        {
            start.date().until((Unit::Year, end.date()))?
        } else {
            start.until((Unit::Year, end))?
        };
        Ok(if absolute { span.abs() } else { span })
    }
    pub(crate) fn day_count(self, start: &Zoned, end: &Zoned) -> Result<u64, Error> {
        if start.time_zone() != end.time_zone() {
            return Ok(
                (end.timestamp().as_second() - start.timestamp().as_second()).unsigned_abs()
                    / 86400,
            );
        }
        let days = u64::from(
            start
                .date()
                .until((Unit::Day, end.date()))?
                .get_days()
                .unsigned_abs(),
        );
        if self <= Self::Day && *start == self.round_date(start)? && *end == self.round_date(end)? {
            return Ok(days);
        }
        let (earlier, later) = if start < end {
            (start, end)
        } else {
            (end, start)
        };
        Ok(days.saturating_sub(u64::from(later.time() < earlier.time())))
    }
    pub(crate) fn shift(self, date: &Zoned, count: i64) -> Result<Zoned, Error> {
        add_interval(date, self.interval().checked_mul(count)?)
    }
    pub(crate) fn parse_date(self, value: &str) -> Result<Zoned, Error> {
        let padded = match self {
            Self::Year => format!("{value}-01-01"),
            Self::Month => format!("{value}-01"),
            Self::Day => value.into(),
            Self::Hour => format!("{value}:00:00"),
            Self::Minute => format!("{value}:00"),
            Self::Second => value.into(),
        };
        padded.into_date()
    }
}

pub(crate) fn add_interval(date: &Zoned, span: Span) -> Result<Zoned, Error> {
    if span.get_years() == 0 && span.get_months() == 0 {
        return Ok(date.checked_add(span)?);
    }
    // PHP calendar intervals overflow short months instead of clamping to their last day.
    let first = date.date().first_of_month();
    let shifted = first
        .checked_add(
            Span::new()
                .years(span.get_years())
                .months(span.get_months()),
        )?
        .checked_add(i64::from(date.day() - 1).days())?;
    let date = resolve_fields(date, shifted.to_datetime(date.time()), true)?;
    Ok(date.checked_add(
        Span::new()
            .days(span.get_days())
            .hours(span.get_hours())
            .minutes(span.get_minutes())
            .seconds(span.get_seconds())
            .milliseconds(span.get_milliseconds())
            .microseconds(span.get_microseconds())
            .nanoseconds(span.get_nanoseconds()),
    )?)
}

pub(crate) fn resolve_fields(
    original: &Zoned,
    dt: DateTime,
    keep_offset: bool,
) -> Result<Zoned, Error> {
    if keep_offset {
        Ok(original
            .with()
            .date(dt.date())
            .time(dt.time())
            .offset_conflict(OffsetConflict::PreferOffset)
            .build()?)
    } else {
        Ok(dt.to_zoned(original.time_zone().clone())?)
    }
}

impl FromStr for Precision {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Error> {
        let precision = match value.len() {
            4 => Self::Year,
            7 => Self::Month,
            10 => Self::Day,
            13 => Self::Hour,
            16 => Self::Minute,
            19 => Self::Second,
            _ => return Err(Error::InvalidPeriod(value.into())),
        };
        precision.parse_date(value)?;
        Ok(precision)
    }
}

impl fmt::Display for Precision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Year => "year",
            Self::Month => "month",
            Self::Day => "day",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::Second => "second",
        })
    }
}
