use crate::{Error, Period};
use jiff::{Span, Zoned, tz::TimeZone};
use std::cmp::Ordering;

/// The calendar duration between a period's included endpoints.
///
/// Unlike [`Period::length`], this does not add one precision unit.
#[derive(Debug, Clone, Copy)]
pub struct PeriodDuration<'a> {
    period: &'a Period,
}

impl<'a> PeriodDuration<'a> {
    /// Takes a view of a period's duration.
    pub fn new(period: &'a Period) -> Self {
        Self { period }
    }
    /// The absolute calendar difference, measured backwards from the included end.
    ///
    /// # Errors
    /// Returns an error if date arithmetic exceeds the supported range.
    pub fn interval(self) -> Result<Span, Error> {
        self.period.precision().diff(
            self.period.included_end(),
            self.period.included_start(),
            true,
        )
    }
    /// Whether endpoints, whole-day count, or calendar durations are equal.
    ///
    /// As in the PHP package, durations can be equal by day count even when their
    /// hours differ. Use [`compare_to`](Self::compare_to) to order calendar durations.
    ///
    /// # Errors
    /// Returns an error if calendar arithmetic exceeds the supported range.
    pub fn equals(self, other: Self) -> Result<bool, Error> {
        if (self.period.start() == other.period.start() && self.period.end() == other.period.end())
            || (self.period.included_start() == other.period.included_start()
                && self.period.included_end() == other.period.included_end())
        {
            return Ok(true);
        }
        let days = |period: &Period| {
            period
                .precision()
                .day_count(period.included_end(), period.included_start())
        };
        Ok(days(self.period)? == days(other.period)? || self.compare_to(other)? == Ordering::Equal)
    }
    /// Orders calendar intervals after adding them to the same current UTC instant.
    ///
    /// # Errors
    /// Returns an error if calendar arithmetic exceeds the supported range.
    pub fn compare_to(self, other: Self) -> Result<Ordering, Error> {
        let now = jiff::Timestamp::from_second(jiff::Timestamp::now().as_second())?
            .to_zoned(TimeZone::UTC);
        self.compare_at(other, &now)
    }
    /// Orders calendar intervals relative to a caller-supplied instant.
    ///
    /// # Errors
    /// Returns an error if calendar arithmetic exceeds the supported range.
    pub fn compare_at(self, other: Self, relative: &Zoned) -> Result<Ordering, Error> {
        Ok(crate::precision::add_interval(relative, self.interval()?)?
            .timestamp()
            .as_second()
            .cmp(
                &crate::precision::add_interval(relative, other.interval()?)?
                    .timestamp()
                    .as_second(),
            ))
    }
    /// Whether this calendar duration is larger.
    ///
    /// # Errors
    /// Returns an error if calendar arithmetic exceeds the supported range.
    pub fn is_larger_than(self, other: Self) -> Result<bool, Error> {
        Ok(self.compare_to(other)? == Ordering::Greater)
    }
    /// Whether this calendar duration is smaller.
    ///
    /// # Errors
    /// Returns an error if calendar arithmetic exceeds the supported range.
    pub fn is_smaller_than(self, other: Self) -> Result<bool, Error> {
        Ok(self.compare_to(other)? == Ordering::Less)
    }
}
