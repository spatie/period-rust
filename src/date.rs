use crate::Error;
use jiff::{
    Timestamp, Zoned,
    civil::{Date, DateTime},
    tz::TimeZone,
};

/// Converts an input date into an owned, timezone-aware date.
///
/// Civil dates and strings without a timezone use UTC. Pass a [`Zoned`] to
/// retain a named timezone. Implement this trait to accept another date type.
pub trait IntoDate {
    /// Resolves the input without changing it.
    ///
    /// # Errors
    /// Returns an error for an invalid date or timezone.
    fn into_date(self) -> Result<Zoned, Error>;
}

impl IntoDate for Zoned {
    fn into_date(self) -> Result<Zoned, Error> {
        Ok(self)
    }
}

impl IntoDate for &Zoned {
    fn into_date(self) -> Result<Zoned, Error> {
        Ok(self.clone())
    }
}

impl IntoDate for Date {
    fn into_date(self) -> Result<Zoned, Error> {
        Ok(self.to_zoned(TimeZone::UTC)?)
    }
}

impl IntoDate for DateTime {
    fn into_date(self) -> Result<Zoned, Error> {
        Ok(self.to_zoned(TimeZone::UTC)?)
    }
}

impl IntoDate for Timestamp {
    fn into_date(self) -> Result<Zoned, Error> {
        Ok(self.to_zoned(TimeZone::UTC))
    }
}

impl IntoDate for &str {
    fn into_date(self) -> Result<Zoned, Error> {
        if self.contains('[') {
            return Ok(self.parse()?);
        }
        if self.len() == 10 {
            return self.parse::<Date>()?.into_date();
        }
        if self.ends_with('Z')
            || self
                .get(10..)
                .is_some_and(|time| time.contains('+') || time.contains('-'))
        {
            let zone = jiff::fmt::temporal::Pieces::parse(self)?
                .to_numeric_offset()
                .map(TimeZone::fixed)
                .unwrap_or(TimeZone::UTC);
            return Ok(self.parse::<Timestamp>()?.to_zoned(zone));
        }
        self.parse::<DateTime>()?.into_date()
    }
}

impl IntoDate for String {
    fn into_date(self) -> Result<Zoned, Error> {
        self.as_str().into_date()
    }
}

impl IntoDate for &String {
    fn into_date(self) -> Result<Zoned, Error> {
        self.as_str().into_date()
    }
}
