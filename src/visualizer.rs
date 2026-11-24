use crate::{Error, PeriodCollection};

/// Draws labeled periods and collections as fixed-width ASCII bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Visualizer {
    width: usize,
}

impl Default for Visualizer {
    fn default() -> Self {
        Self::new(60)
    }
}

impl Visualizer {
    /// Sets the bar width, excluding labels. A zero width produces labels only.
    pub const fn new(width: usize) -> Self {
        Self { width }
    }
    /// Visualizes rows. Wrap a single period in a one-entry collection.
    ///
    /// As in the PHP visualizer, bars use original endpoints and whole Unix seconds,
    /// with a floor for the start and a ceiling for the end.
    ///
    /// # Errors
    /// Returns an error if the outer boundaries cannot form a period.
    pub fn visualize(&self, blocks: &[(&str, PeriodCollection)]) -> Result<String, Error> {
        let periods = PeriodCollection::new(
            blocks
                .iter()
                .flat_map(|(_, periods)| periods.iter().cloned()),
        );
        let bounds = periods.boundaries()?;
        let name_length = blocks.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        let mut lines = Vec::with_capacity(blocks.len());
        for (name, periods) in blocks {
            let mut row = vec![false; self.width];
            if let Some(bounds) = &bounds {
                let start = i128::from(bounds.start().timestamp().as_second());
                let length = i128::from(bounds.end().timestamp().as_second()) - start;
                for period in periods {
                    if length == 0 {
                        if let Some(cell) = row.first_mut() {
                            *cell = true;
                        }
                        continue;
                    }
                    let from = (i128::from(period.start().timestamp().as_second()) - start)
                        * self.width as i128;
                    let to = (i128::from(period.end().timestamp().as_second()) - start)
                        * self.width as i128;
                    let from = from.div_euclid(length);
                    let to = -(-to).div_euclid(length);
                    for (i, cell) in row.iter_mut().enumerate() {
                        if from <= i as i128 && (i as i128) < to {
                            *cell = true;
                        }
                    }
                }
            }
            let bars: String = row
                .iter()
                .enumerate()
                .map(|(i, &cell)| {
                    let previous = i > 0 && row[i - 1];
                    let next = row.get(i + 1).copied().unwrap_or(false);
                    match (cell, previous, next) {
                        (false, _, _) => ' ',
                        (true, false, true) => '[',
                        (true, true, false) => ']',
                        _ => '=',
                    }
                })
                .collect();
            lines.push(format!("{name:<name_length$}    {bars}"));
        }
        Ok(lines.join("\n"))
    }
}
