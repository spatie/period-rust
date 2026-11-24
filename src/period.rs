use crate::{Boundaries, Error, IntoDate, PeriodCollection, PeriodDuration, Precision};
use jiff::Zoned;
use std::{
    fmt,
    hash::{Hash, Hasher},
    iter::FusedIterator,
    str::FromStr,
};

/// An immutable date interval with explicit precision and endpoint inclusion.
///
/// Use [`make`](Self::make) for inclusive days or [`with_options`](Self::with_options)
/// to choose precision and boundaries. Both round input dates down; [`new`](Self::new)
/// preserves unrounded dates.
#[derive(Debug, Clone)]
pub struct Period {
    start: Zoned,
    end: Zoned,
    included_start: Zoned,
    included_end: Zoned,
    precision: Precision,
    boundaries: Boundaries,
}

impl Period {
    /// Creates an inclusive period at day precision.
    ///
    /// ```
    /// # use spatie_period::Period;
    /// let period = Period::make("2021-01-01", "2021-01-31")?;
    /// assert_eq!(period.length()?, 31);
    /// # Ok::<(), spatie_period::Error>(())
    /// ```
    ///
    /// # Errors
    /// Returns an error for invalid dates, reversed endpoints, or out-of-range arithmetic.
    pub fn make(start: impl IntoDate, end: impl IntoDate) -> Result<Self, Error> {
        Self::with_options(start, end, Precision::Day, Boundaries::ExcludeNone)
    }

    /// Creates a period, rounding both dates down to the selected precision.
    ///
    /// # Errors
    /// Returns an error for invalid dates, reversed endpoints, or out-of-range arithmetic.
    pub fn with_options(
        start: impl IntoDate,
        end: impl IntoDate,
        precision: Precision,
        boundaries: Boundaries,
    ) -> Result<Self, Error> {
        Self::new(
            precision.round_date(&start.into_date()?)?,
            precision.round_date(&end.into_date()?)?,
            precision,
            boundaries,
        )
    }

    /// Constructs a period without rounding its dates.
    ///
    /// # Errors
    /// Returns an error for reversed endpoints or out-of-range boundary arithmetic.
    pub fn new(
        start: Zoned,
        end: Zoned,
        precision: Precision,
        boundaries: Boundaries,
    ) -> Result<Self, Error> {
        if start > end {
            return Err(Error::EndBeforeStart);
        }
        let included_start = if boundaries.start_included() {
            start.clone()
        } else if precision <= Precision::Day && start == precision.round_date(&start)? {
            precision.increment(&start)?
        } else {
            precision.shift(&start, 1)?
        };
        let included_end = if boundaries.end_included() {
            end.clone()
        } else if precision <= Precision::Day && end == precision.round_date(&end)? {
            precision.decrement(&end)?
        } else {
            precision.shift(&end, -1)?
        };
        Ok(Self {
            start,
            end,
            included_start,
            included_end,
            precision,
            boundaries,
        })
    }

    /// Parses dates using a strftime-style format, then rounds to the precision.
    ///
    /// # Errors
    /// Returns an error when dates do not match the format or cannot form a period.
    pub fn with_format(
        start: &str,
        end: &str,
        precision: Precision,
        boundaries: Boundaries,
        format: &str,
    ) -> Result<Self, Error> {
        fn parse(value: &str, format: &str) -> Result<Zoned, Error> {
            let fields = jiff::fmt::strtime::BrokenDownTime::parse(format, value)?;
            if fields.offset().is_some()
                || fields.iana_time_zone().is_some()
                || fields.timestamp().is_some()
            {
                return Ok(fields.to_zoned()?);
            }
            fields.to_datetime()?.into_date()
        }
        Self::with_options(
            parse(start, format)?,
            parse(end, format)?,
            precision,
            boundaries,
        )
    }

    /// Constructs a period from its included endpoints, retaining boundary style.
    ///
    /// # Errors
    /// Returns an error for invalid dates, reversed endpoints, or out-of-range arithmetic.
    pub fn from_included(
        start: impl IntoDate,
        end: impl IntoDate,
        precision: Precision,
        boundaries: Boundaries,
    ) -> Result<Self, Error> {
        let start = precision.round_date(&start.into_date()?)?;
        let end = precision.round_date(&end.into_date()?)?;
        Self::new(
            boundaries.real_start(&start, precision)?,
            boundaries.real_end(&end, precision)?,
            precision,
            boundaries,
        )
    }

    /// The original (possibly excluded) start.
    pub fn start(&self) -> &Zoned {
        &self.start
    }
    /// The original (possibly excluded) end.
    pub fn end(&self) -> &Zoned {
        &self.end
    }
    /// The first included date.
    pub fn included_start(&self) -> &Zoned {
        &self.included_start
    }
    /// The last included date.
    pub fn included_end(&self) -> &Zoned {
        &self.included_end
    }
    /// The period's precision.
    pub fn precision(&self) -> Precision {
        self.precision
    }
    /// Which endpoints are excluded.
    pub fn boundaries(&self) -> Boundaries {
        self.boundaries
    }
    /// Whether the original start is included.
    pub fn is_start_included(&self) -> bool {
        self.boundaries.start_included()
    }
    /// Whether the original start is excluded.
    pub fn is_start_excluded(&self) -> bool {
        self.boundaries.start_excluded()
    }
    /// Whether the original end is included.
    pub fn is_end_included(&self) -> bool {
        self.boundaries.end_included()
    }
    /// Whether the original end is excluded.
    pub fn is_end_excluded(&self) -> bool {
        self.boundaries.end_excluded()
    }

    /// Returns the latest second of the included end at the requested precision.
    /// `None` uses the period's precision.
    ///
    /// # Errors
    /// Returns an error if the requested precision is finer than the period or the date is out of range.
    pub fn ceiling_end(&self, precision: Option<Precision>) -> Result<Zoned, Error> {
        let precision = precision.unwrap_or(self.precision);
        if precision > self.precision {
            return Err(Error::InvalidCeiling {
                period: self.precision,
                ceiling: precision,
            });
        }
        precision.ceil_date(&self.included_end)
    }

    /// Counts precision units, including both included boundaries.
    ///
    /// Month and year lengths count iterator entries. Days use calendar differences;
    /// hours, minutes and seconds use elapsed time. Across timezone transitions this
    /// need not equal the iterator count.
    ///
    /// # Errors
    /// Returns an error if calendar arithmetic exceeds the supported range.
    pub fn length(&self) -> Result<u64, Error> {
        if self.precision <= Precision::Month {
            return self.iter().try_fold(0, |n, date| date.map(|_| n + 1));
        }
        if self.included_start > self.included_end && self.iter().next().is_none() {
            return Ok(0);
        }
        if self.precision >= Precision::Hour {
            let seconds = (self.included_end.timestamp().as_second()
                - self.included_start.timestamp().as_second())
            .unsigned_abs();
            return Ok(seconds
                / match self.precision {
                    Precision::Hour => 3600,
                    Precision::Minute => 60,
                    _ => 1,
                }
                + 1);
        }
        Ok(self
            .precision
            .day_count(&self.included_start, &self.included_end)?
            + 1)
    }

    /// The calendar duration between the included endpoints (not the unit count).
    pub fn duration(&self) -> PeriodDuration<'_> {
        PeriodDuration::new(self)
    }
    /// Iterates dates by the period's calendar interval.
    pub fn iter(&self) -> Dates {
        Dates {
            next: Some(self.included_start.clone()),
            end: self.included_end.clone(),
            precision: self.precision,
            error: None,
        }
    }

    /// Whether the included start precedes a date rounded to this precision.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn starts_before(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_start < self.precision.round_date(&date.into_date()?)?)
    }
    /// Whether the included start precedes or equals a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn starts_before_or_at(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_start <= self.precision.round_date(&date.into_date()?)?)
    }
    /// Whether the included start follows a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn starts_after(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_start > self.precision.round_date(&date.into_date()?)?)
    }
    /// Whether the included start follows or equals a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn starts_after_or_at(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_start >= self.precision.round_date(&date.into_date()?)?)
    }
    /// Whether the included start has the same Unix second as a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn starts_at(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_start.timestamp().as_second()
            == self
                .precision
                .round_date(&date.into_date()?)?
                .timestamp()
                .as_second())
    }
    /// Whether the included end precedes a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn ends_before(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_end < self.precision.round_date(&date.into_date()?)?)
    }
    /// Whether the included end precedes or equals a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn ends_before_or_at(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_end <= self.precision.round_date(&date.into_date()?)?)
    }
    /// Whether the included end follows a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn ends_after(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_end > self.precision.round_date(&date.into_date()?)?)
    }
    /// Whether the included end follows or equals a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn ends_after_or_at(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_end >= self.precision.round_date(&date.into_date()?)?)
    }
    /// Whether the included end has the same Unix second as a rounded date.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn ends_at(&self, date: impl IntoDate) -> Result<bool, Error> {
        Ok(self.included_end.timestamp().as_second()
            == self
                .precision
                .round_date(&date.into_date()?)?
                .timestamp()
                .as_second())
    }
    /// Whether a rounded date is within the included endpoints.
    ///
    /// # Errors
    /// Returns an error for an invalid or out-of-range date.
    pub fn contains(&self, date: impl IntoDate) -> Result<bool, Error> {
        let date = self.precision.round_date(&date.into_date()?)?;
        Ok(self.included_start <= date && self.included_end >= date)
    }
    /// Whether another period's included endpoints lie within this period.
    /// Unlike overlap, containment permits different precisions.
    pub fn contains_period(&self, other: &Self) -> bool {
        self.included_start <= other.included_start && self.included_end >= other.included_end
    }
    /// Whether the included endpoints match, independent of boundary style.
    ///
    /// # Errors
    /// Returns [`Error::PrecisionMismatch`] for different precisions.
    pub fn equals(&self, other: &Self) -> Result<bool, Error> {
        self.ensure_precision(other)?;
        Ok(self == other)
    }
    /// Whether two periods have included dates in common.
    ///
    /// # Errors
    /// Returns [`Error::PrecisionMismatch`] for different precisions.
    pub fn overlaps_with(&self, other: &Self) -> Result<bool, Error> {
        self.ensure_precision(other)?;
        Ok(self.included_start <= other.included_end && other.included_start <= self.included_end)
    }
    /// Whether the periods are adjacent but do not overlap.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn touches_with(&self, other: &Self) -> Result<bool, Error> {
        self.ensure_precision(other)?;
        if self.included_end < other.included_start {
            return Ok(self.precision.increment(&self.included_end)?
                == other.precision.round_date(&other.included_start)?);
        }
        if other.included_end < self.included_start {
            return Ok(other.precision.increment(&other.included_end)?
                == self.precision.round_date(&self.included_start)?);
        }
        Ok(false)
    }
    /// Finds the gap, or `None` if the periods overlap or touch.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn gap(&self, other: &Self) -> Result<Option<Self>, Error> {
        if self.overlaps_with(other)? || self.touches_with(other)? {
            return Ok(None);
        }
        let (before, after) = if self.included_start >= other.included_end {
            (other, self)
        } else {
            (self, other)
        };
        Ok(Some(Self::with_options(
            self.precision.shift(&before.included_end, 1)?,
            self.precision.shift(&after.included_start, -1)?,
            self.precision,
            Boundaries::ExcludeNone,
        )?))
    }
    /// Intersects this period with every other period. No others yields `None`.
    /// The result retains this period's boundary style.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn overlap(&self, others: &[Self]) -> Result<Option<Self>, Error> {
        if others.is_empty() {
            return Ok(None);
        }
        let mut overlap = self.clone();
        for other in others {
            overlap.ensure_precision(other)?;
            let start = overlap.included_start.max(other.included_start.clone());
            let end = if overlap.included_end < other.included_end {
                overlap.included_end
            } else {
                other.included_end.clone()
            };
            if start > end {
                return Ok(None);
            }
            overlap = Self::from_included(start, end, self.precision, self.boundaries)?;
        }
        Ok(Some(overlap))
    }
    /// Collects each pairwise overlap, preserving input order and duplicates.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn overlap_any(&self, others: &[Self]) -> Result<PeriodCollection, Error> {
        let mut result = PeriodCollection::default();
        for other in others {
            if let Some(overlap) = self.overlap(std::slice::from_ref(other))? {
                result.push(overlap);
            }
        }
        Ok(result)
    }
    /// Subtracts the other periods. No others returns a collection containing this period.
    /// Results retain this period's boundary style.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn subtract(&self, others: &[Self]) -> Result<PeriodCollection, Error> {
        if others.len() > 1 {
            let subtractions = others
                .iter()
                .map(|other| self.subtract(std::slice::from_ref(other)))
                .collect::<Result<Vec<_>, _>>()?;
            return PeriodCollection::new([self.clone()]).overlap_all(&subtractions);
        }
        let mut remaining = PeriodCollection::new([self.clone()]);
        for other in others {
            self.ensure_precision(other)?;
            let mut next = PeriodCollection::default();
            for part in remaining {
                if !part.overlaps_with(other)? {
                    next.push(part);
                    continue;
                }
                if part.included_start < other.included_start {
                    next.push(Self::from_included(
                        &part.included_start,
                        self.precision.shift(&other.included_start, -1)?,
                        self.precision,
                        self.boundaries,
                    )?);
                }
                if part.included_end > other.included_end {
                    next.push(Self::from_included(
                        self.precision.shift(&other.included_end, 1)?,
                        &part.included_end,
                        self.precision,
                        self.boundaries,
                    )?);
                }
            }
            remaining = next;
        }
        Ok(remaining)
    }
    /// Symmetric difference: dates in either period but not both.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn diff_symmetric(&self, other: &Self) -> Result<PeriodCollection, Error> {
        if !self.overlaps_with(other)? {
            return Ok(PeriodCollection::new([self.clone(), other.clone()]));
        }
        let bounds = Self::with_options(
            self.included_start
                .clone()
                .min(other.included_start.clone()),
            if self.included_end < other.included_end {
                other.included_end.clone()
            } else {
                self.included_end.clone()
            },
            self.precision,
            Boundaries::ExcludeNone,
        )?;
        let overlap = self.overlap(std::slice::from_ref(other))?.ok_or_else(|| {
            Error::InvalidPeriod("No included overlap for symmetric difference".into())
        })?;
        bounds.subtract(&[overlap])
    }
    /// Creates a period after this one using the same calendar duration and boundary style.
    ///
    /// # Errors
    /// Returns an error if calendar arithmetic exceeds the supported range.
    pub fn renew(&self) -> Result<Self, Error> {
        let length = self
            .precision
            .diff(&self.included_start, &self.included_end, false)?;
        let start = self.precision.shift(&self.included_end, 1)?;
        Self::with_options(
            &start,
            crate::precision::add_interval(&start, length)?,
            self.precision,
            self.boundaries,
        )
    }
    fn ensure_precision(&self, other: &Self) -> Result<(), Error> {
        if self.precision == other.precision {
            Ok(())
        } else {
            Err(Error::PrecisionMismatch)
        }
    }
}

impl PartialEq for Period {
    fn eq(&self, other: &Self) -> bool {
        self.precision == other.precision
            && self.included_start.timestamp().as_second()
                == other.included_start.timestamp().as_second()
            && self.included_end.timestamp().as_second()
                == other.included_end.timestamp().as_second()
    }
}
impl Eq for Period {}
impl Hash for Period {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.precision.hash(state);
        self.included_start.timestamp().as_second().hash(state);
        self.included_end.timestamp().as_second().hash(state);
    }
}
impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{},{}{}",
            if self.is_start_included() { '[' } else { '(' },
            self.start.strftime(self.precision.date_format()),
            self.end.strftime(self.precision.date_format()),
            if self.is_end_included() { ']' } else { ')' }
        )
    }
}
impl FromStr for Period {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Error> {
        let invalid = || Error::InvalidPeriod(value.into());
        let value = value.trim();
        let first = value.chars().next().ok_or_else(invalid)?;
        let last = value.chars().last().ok_or_else(invalid)?;
        let boundaries = format!("{first}{last}").parse()?;
        let inner = value
            .get(1..value.len().saturating_sub(1))
            .ok_or_else(invalid)?;
        let (start, end) = inner.split_once(',').ok_or_else(invalid)?;
        let (start, end) = (start.trim(), end.trim());
        let precision: Precision = start.parse()?;
        if end.parse::<Precision>()? != precision {
            return Err(invalid());
        }
        Self::new(
            precision.parse_date(start)?,
            precision.parse_date(end)?,
            precision,
            boundaries,
        )
    }
}

/// An owned iterator over a period's dates.
///
/// Like PHP's `DatePeriod`, iteration advances calendar fields, so it can skip
/// the second occurrence of a repeated hour. Arithmetic overflow is yielded once
/// as an error, after which the iterator is exhausted.
#[derive(Debug)]
pub struct Dates {
    next: Option<Zoned>,
    end: Zoned,
    precision: Precision,
    error: Option<Error>,
}
impl Iterator for Dates {
    type Item = Result<Zoned, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(error) = self.error.take() {
            return Some(Err(error));
        }
        let current = self.next.take()?;
        if current.timestamp().as_nanosecond()
            >= self.end.timestamp().as_nanosecond() + 1_000_000_000
        {
            return None;
        }
        // Do not compute a successor outside the representable range when already at the end.
        if current < self.end {
            let next = if self.precision >= Precision::Hour {
                current
                    .datetime()
                    .checked_add(self.precision.interval())
                    .map_err(Error::from)
                    .and_then(|dt| crate::precision::resolve_fields(&current, dt, true))
            } else {
                self.precision.shift(&current, 1)
            };
            match next {
                Ok(next) => self.next = Some(next),
                Err(error) => self.error = Some(error),
            }
        }
        Some(Ok(current))
    }
}
impl FusedIterator for Dates {}
impl IntoIterator for &Period {
    type Item = Result<Zoned, Error>;
    type IntoIter = Dates;
    fn into_iter(self) -> Dates {
        self.iter()
    }
}
impl IntoIterator for Period {
    type Item = Result<Zoned, Error>;
    type IntoIter = Dates;
    fn into_iter(self) -> Dates {
        self.iter()
    }
}
