#![doc = include_str!("../README.md")]

mod boundaries;
mod collection;
mod date;
mod duration;
mod error;
mod period;
mod precision;
mod visualizer;

pub use boundaries::Boundaries;
pub use collection::PeriodCollection;
pub use date::IntoDate;
pub use duration::PeriodDuration;
pub use error::Error;
pub use period::{Dates, Period};
pub use precision::Precision;
pub use visualizer::Visualizer;

/// Date and time types used by this crate.
pub use jiff;

#[cfg(doctest)]
mod guide {
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}
