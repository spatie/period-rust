use crate::{Error, Precision};
use jiff::Zoned;
use std::str::FromStr;

/// Which endpoints are excluded from a period.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Boundaries {
    /// Include both endpoints: `[]`.
    #[default]
    ExcludeNone,
    /// Exclude the start: `(]`.
    ExcludeStart,
    /// Exclude the end: `[)`.
    ExcludeEnd,
    /// Exclude both endpoints: `()`.
    ExcludeAll,
}

impl Boundaries {
    /// All four boundary combinations.
    pub const ALL: [Self; 4] = [
        Self::ExcludeNone,
        Self::ExcludeStart,
        Self::ExcludeEnd,
        Self::ExcludeAll,
    ];
    /// Whether the start is excluded.
    pub const fn start_excluded(self) -> bool {
        matches!(self, Self::ExcludeStart | Self::ExcludeAll)
    }
    /// Whether the start is included.
    pub const fn start_included(self) -> bool {
        !self.start_excluded()
    }
    /// Whether the end is excluded.
    pub const fn end_excluded(self) -> bool {
        matches!(self, Self::ExcludeEnd | Self::ExcludeAll)
    }
    /// Whether the end is included.
    pub const fn end_included(self) -> bool {
        !self.end_excluded()
    }
    /// Recovers the original start from an included start.
    ///
    /// # Errors
    /// Returns an error if date arithmetic exceeds the supported range.
    pub fn real_start(self, included_start: &Zoned, precision: Precision) -> Result<Zoned, Error> {
        if self.start_included() {
            Ok(included_start.clone())
        } else {
            precision.decrement(included_start)
        }
    }
    /// Recovers the original end from an included end.
    ///
    /// # Errors
    /// Returns an error if date arithmetic exceeds the supported range.
    pub fn real_end(self, included_end: &Zoned, precision: Precision) -> Result<Zoned, Error> {
        if self.end_included() {
            Ok(included_end.clone())
        } else {
            precision.increment(included_end)
        }
    }
}

impl FromStr for Boundaries {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Error> {
        match value {
            "[]" => Ok(Self::ExcludeNone),
            "(]" => Ok(Self::ExcludeStart),
            "[)" => Ok(Self::ExcludeEnd),
            "()" => Ok(Self::ExcludeAll),
            _ => Err(Error::InvalidPeriod(value.into())),
        }
    }
}
