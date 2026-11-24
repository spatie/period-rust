use crate::{Boundaries, Error, Period};
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    slice,
};

/// An ordered collection of periods. Operations return new collections and keep duplicates.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PeriodCollection {
    periods: Vec<Period>,
}

impl PeriodCollection {
    /// Collects periods in their input order.
    pub fn new(periods: impl IntoIterator<Item = Period>) -> Self {
        periods.into_iter().collect()
    }
    /// Number of periods, not the sum of their lengths.
    pub fn len(&self) -> usize {
        self.periods.len()
    }
    /// Whether there are no periods.
    pub fn is_empty(&self) -> bool {
        self.periods.is_empty()
    }
    /// Borrows the periods in order.
    pub fn iter(&self) -> slice::Iter<'_, Period> {
        self.periods.iter()
    }
    /// Mutably borrows the collection's entries.
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, Period> {
        self.periods.iter_mut()
    }
    /// Gets a period without panicking for a missing index.
    pub fn get(&self, index: usize) -> Option<&Period> {
        self.periods.get(index)
    }
    /// Appends a period to this collection.
    pub fn push(&mut self, period: Period) {
        self.periods.push(period);
    }
    /// Removes a period, shifting following entries left. Missing indices return `None`.
    pub fn remove(&mut self, index: usize) -> Option<Period> {
        if index < self.len() {
            Some(self.periods.remove(index))
        } else {
            None
        }
    }
    /// Copies the collection with additional periods appended.
    pub fn add(&self, periods: impl IntoIterator<Item = Period>) -> Self {
        let mut result = self.clone();
        result.extend(periods);
        result
    }
    /// Maps periods into a new collection.
    pub fn map(&self, map: impl FnMut(&Period) -> Period) -> Self {
        self.iter().map(map).collect()
    }
    /// Folds periods in input order, starting with `initial`.
    pub fn reduce<T>(&self, initial: T, reduce: impl FnMut(T, &Period) -> T) -> T {
        self.iter().fold(initial, reduce)
    }
    /// Keeps matching periods, in input order.
    pub fn filter(&self, mut predicate: impl FnMut(&Period) -> bool) -> Self {
        self.iter().filter(|p| predicate(p)).cloned().collect()
    }
    /// Stable sort by included start, leaving the original collection unchanged.
    pub fn sort(&self) -> Self {
        let mut result = self.clone();
        result
            .periods
            .sort_by(|a, b| a.included_start().cmp(b.included_start()));
        result
    }
    /// Removes duplicate string representations, keeping the last value at its first position.
    pub fn unique(&self) -> Self {
        let mut positions = HashMap::new();
        let mut result = Self::default();
        for period in self {
            let index = *positions.entry(period.to_string()).or_insert(result.len());
            if index == result.len() {
                result.push(period.clone());
            } else {
                result[index] = period.clone();
            }
        }
        result
    }
    /// The outer included boundaries, at the first period's precision. Empty collections yield `None`.
    ///
    /// # Errors
    /// Returns an error if date arithmetic exceeds the supported range.
    pub fn boundaries(&self) -> Result<Option<Period>, Error> {
        let Some(first) = self.periods.first() else {
            return Ok(None);
        };
        let mut start = first.included_start();
        let mut end = first.included_end();
        for period in self {
            start = start.min(period.included_start());
            if end < period.included_end() {
                end = period.included_end();
            }
        }
        Ok(Some(Period::with_options(
            start,
            end,
            first.precision(),
            Boundaries::ExcludeNone,
        )?))
    }
    /// Gaps between the periods within their outer boundaries.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn gaps(&self) -> Result<Self, Error> {
        match self.boundaries()? {
            Some(bounds) => bounds.subtract(self.as_ref()),
            None => Ok(Self::default()),
        }
    }
    /// Intersects the given period with each entry. Results use its boundary style.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn intersect(&self, intersection: &Period) -> Result<Self, Error> {
        intersection.overlap_any(self.as_ref())
    }
    /// Subtracts periods from each entry, preserving order, duplicates and boundary style.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn subtract(&self, others: &[Period]) -> Result<Self, Error> {
        let mut result = Self::default();
        for period in self {
            result.extend(period.subtract(others)?);
        }
        Ok(result)
    }
    /// Pairwise intersections with each collection in turn. No others returns a copy.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn overlap_all(&self, others: &[Self]) -> Result<Self, Error> {
        let mut result = self.clone();
        for other in others {
            let mut next = Self::default();
            for period in &result {
                next.extend(period.overlap_any(other.as_ref())?);
            }
            result = next;
        }
        Ok(result)
    }
    /// Merges overlapping and touching ranges into inclusive periods in chronological order.
    ///
    /// # Errors
    /// Returns an error for different precisions or out-of-range arithmetic.
    pub fn union(&self) -> Result<Self, Error> {
        let Some(bounds) = self.boundaries()? else {
            return Ok(Self::default());
        };
        bounds.subtract(bounds.subtract(self.as_ref())?.as_ref())
    }
}

impl AsRef<[Period]> for PeriodCollection {
    fn as_ref(&self) -> &[Period] {
        &self.periods
    }
}
impl From<Vec<Period>> for PeriodCollection {
    fn from(periods: Vec<Period>) -> Self {
        Self { periods }
    }
}
impl<const N: usize> From<[Period; N]> for PeriodCollection {
    fn from(periods: [Period; N]) -> Self {
        Self::new(periods)
    }
}
impl FromIterator<Period> for PeriodCollection {
    fn from_iter<T: IntoIterator<Item = Period>>(iter: T) -> Self {
        Self {
            periods: iter.into_iter().collect(),
        }
    }
}
impl Extend<Period> for PeriodCollection {
    fn extend<T: IntoIterator<Item = Period>>(&mut self, iter: T) {
        self.periods.extend(iter);
    }
}
impl Index<usize> for PeriodCollection {
    type Output = Period;
    fn index(&self, index: usize) -> &Period {
        &self.periods[index]
    }
}
impl IndexMut<usize> for PeriodCollection {
    fn index_mut(&mut self, index: usize) -> &mut Period {
        &mut self.periods[index]
    }
}
impl IntoIterator for PeriodCollection {
    type Item = Period;
    type IntoIter = std::vec::IntoIter<Period>;
    fn into_iter(self) -> Self::IntoIter {
        self.periods.into_iter()
    }
}
impl<'a> IntoIterator for &'a PeriodCollection {
    type Item = &'a Period;
    type IntoIter = slice::Iter<'a, Period>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &'a mut PeriodCollection {
    type Item = &'a mut Period;
    type IntoIter = slice::IterMut<'a, Period>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
