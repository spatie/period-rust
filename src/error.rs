use crate::Precision;

/// A period could not be constructed, compared, or transformed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The end precedes the start.
    #[error("The end time is before the start time.")]
    EndBeforeStart,
    /// An interval string or boundary pair is malformed.
    #[error("Invalid period: {0}")]
    InvalidPeriod(String),
    /// Operations require matching precisions.
    #[error("Cannot compare periods with different precisions.")]
    PrecisionMismatch,
    /// A ceiling cannot be finer than the period's precision.
    #[error("Cannot get the latest {period} of a {ceiling}.")]
    InvalidCeiling {
        /// The period's precision.
        period: Precision,
        /// The requested ceiling precision.
        ceiling: Precision,
    },
    /// A date is invalid or calendar arithmetic exceeded the supported range.
    #[error(transparent)]
    Date(#[from] jiff::Error),
}
