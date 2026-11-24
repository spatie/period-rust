---
title: Usage
weight: 4
---

## Creating periods

You're encouraged to create periods using their static constructor:

```rust
# use spatie_period::Period;
# fn main() -> Result<(), spatie_period::Error> {
let period = Period::make("2021-01-01", "2021-01-31")?;
# assert_eq!(period.length()?, 31);
# Ok(())
# }
```

You can manually construct a period, but you'll need to manually provide its **precision** and **boundaries**. Using `Period::make`, the default precision (`Precision::Day`) and default boundaries (`Boundaries::ExcludeNone`) are used.

Before discussing the API provided by this package, it's important to understand both how precision and boundaries are used.

### Precision

Date precision is of utmost importance if you want to reliably compare two periods.
The following example:

> Given two periods: `[2021-01-01, 2021-01-15]` and `[2021-01-15, 2021-01-31]`; do they overlap?

At first glance the answer is "yes": they overlap on `2021-01-15`.
But what if the first period ends at `2021-01-15 10:00:00`,
while the second starts at `2021-01-15 15:00:00`?
Now they don't anymore!

This is why this package requires you to specify a precision with each period.
Only periods with the same precision can be compared.

A more in-depth explanation on why precision is so important can be found [here](https://stitcher.io/blog/comparing-dates).
A period's precision can be specified when constructing that period:

```rust
# use spatie_period::{Period, Precision, Boundaries};
# fn main() -> Result<(), spatie_period::Error> {
Period::with_options("2021-01-01", "2021-02-01", Precision::Day, Boundaries::ExcludeNone)?;
# Ok(())
# }
```

The default precision is set on days. These are the available precision options:

```rust
# use spatie_period::Precision;
Precision::Year;
Precision::Month;
Precision::Day;
Precision::Hour;
Precision::Minute;
Precision::Second;
```

### Boundaries

By default, period comparisons are done with included boundaries.
This means that these two periods overlap:

```rust
# use spatie_period::Period;
# fn main() -> Result<(), spatie_period::Error> {
let a = Period::make("2021-01-01", "2021-02-01")?;
let b = Period::make("2021-02-01", "2021-02-28")?;

a.overlaps_with(&b)?; // true
# assert!(a.overlaps_with(&b)?);
# Ok(())
# }
```

The length of a period will also include both boundaries:

```rust
# use spatie_period::Period;
# fn main() -> Result<(), spatie_period::Error> {
let a = Period::make("2021-01-01", "2021-01-31")?;

a.length()?; // 31
# assert_eq!(a.length()?, 31);
# Ok(())
# }
```

It's possible to override the boundary behaviour:

```rust
# use spatie_period::{Period, Precision, Boundaries};
# fn main() -> Result<(), spatie_period::Error> {
let a = Period::with_options("2021-01-01", "2021-02-01", Precision::Day, Boundaries::ExcludeEnd)?;
let b = Period::with_options("2021-02-01", "2021-02-28", Precision::Day, Boundaries::ExcludeEnd)?;

a.overlaps_with(&b)?; // false
# assert!(!a.overlaps_with(&b)?);
# Ok(())
# }
```

There are four types of boundary exclusion:

```rust
# use spatie_period::Boundaries;
Boundaries::ExcludeNone;
Boundaries::ExcludeStart;
Boundaries::ExcludeEnd;
Boundaries::ExcludeAll;
```

For day, hour, minute, and second precision, `length()` returns `0` when boundary exclusions move the included start after the included end and the iterator yields no dates:

```rust
# use spatie_period::{Period, Precision, Boundaries};
# fn main() -> Result<(), spatie_period::Error> {
let empty = Period::with_options("2021-01-01", "2021-01-01", Precision::Day, Boundaries::ExcludeAll)?;

empty.length()?; // 0
# assert_eq!(empty.length()?, 0);
# Ok(())
# }
```

Month and year lengths continue to use the period's iterator. Other lengths retain their existing arithmetic when dates are yielded, including manually constructed periods with fractional seconds. Length and iterator count are not generally interchangeable across daylight-saving transitions.

## Reference

The `Period` struct offers a rich API to interact and compare with other periods and collections of periods. Take into account that only periods with the same precision can be compared. Start and end date comparisons use the period's precision and its included boundaries:

- `starts_before(date)`: whether a period starts before a given date.
- `starts_before_or_at(date)`: whether a period starts before or at a given date.
- `starts_after(date)`: whether a period starts after a given date.
- `starts_after_or_at(date)`: whether a period starts after or at a given date.
- `starts_at(date)`: whether a period starts at a given date.
- `ends_before(date)`: whether a period ends before a given date.
- `ends_before_or_at(date)`: whether a period ends before or at a given date.
- `ends_after(date)`: whether a period ends after a given date.
- `ends_after_or_at(date)`: whether a period ends after or at a given date.
- `ends_at(date)`: whether a period ends at a given date.
- `overlaps_with(&period)`: whether a period overlaps with another period.
- `touches_with(&other)`: whether a period touches with another period.
- `contains(date)`: whether a period contains a single date.
- `contains_period(&other)`: whether a period contains another period. This comparison permits different precisions.
- `equals(&period)`: whether a period equals another period.

These comparisons return `Result<bool, Error>`, except `contains_period`, which returns `bool`. Use `?` to propagate invalid dates and precision mismatches. `==` compares included endpoints and precision without returning an error; different precisions are unequal.

---

On top of comparisons, the `Period` struct also offers a bunch of operations:

### `overlap(&[Period]) -> Result<Option<Period>, Error>`

Overlaps two or more periods on each other. The resulting period is the intersection of all periods combined.

![Overlap](/docs/period-rust/v1/img/period-overlap.png)

### `overlap_any(&[Period]) -> Result<PeriodCollection, Error>`

Overlaps two or more periods on each other. Whenever two periods overlap, that overlapping period is added to a collection which will be returned as the final result.

![Overlap any](/docs/period-rust/v1/img/period-overlap-any.png)

### `subtract(&[Period]) -> Result<PeriodCollection, Error>`

Subtracts one or more periods from the main period. This is the inverse operation of overlap.

![Subtract](/docs/period-rust/v1/img/period-subtract.png)

### `gap(&Period) -> Result<Option<Period>, Error>`

Gets the gap between two periods, or `None` if the periods overlap or touch.

![Gap](/docs/period-rust/v1/img/period-gap.png)

### `diff_symmetric(&Period) -> Result<PeriodCollection, Error>`

Performs a [symmetric diff](https://www.math-only-math.com/symmetric-difference-using-Venn-diagram.html) between two periods.

![Symmetric difference](/docs/period-rust/v1/img/period-diff-symmetric.png)

### `renew() -> Result<Period, Error>`

Renew the current period, creating a new period with the same calendar duration that happens _after_ the current period. The new period keeps the original boundary style, so excluded boundaries can change its included length.

![Renew](/docs/period-rust/v1/img/period-renew.png)

---

Next, the `Period` struct also has some getters:

- `is_start_included() -> bool`
- `is_start_excluded() -> bool`
- `is_end_included() -> bool`
- `is_end_excluded() -> bool`
- `start() -> &Zoned`
- `included_start() -> &Zoned`
- `end() -> &Zoned`
- `included_end() -> &Zoned`
- `ceiling_end(Option<Precision>) -> Result<Zoned, Error>`: pass `None` for the period's precision. A finer precision returns an error.
- `length() -> Result<u64, Error>`
- `duration() -> PeriodDuration<'_>`
- `precision() -> Precision`
- `boundaries() -> Boundaries`

`duration()` measures the calendar interval between included endpoints, without the extra unit counted by `length()`. Use `equals`, `is_larger_than`, `is_smaller_than` or `compare_to` to compare durations. As in the PHP package, `equals` also accepts equal whole-day counts even when the hours differ. `compare_to` returns `std::cmp::Ordering` after adding both calendar intervals to the same current UTC second. `compare_at` accepts an explicit reference date.

Use `to_string()` for the original endpoints with boundary notation, such as `[2021-01-01,2021-01-31]`. Parse that notation with `str::parse::<Period>()`; the date format determines precision. The notation has no timezone, so parsing it uses UTC.

---

The `PeriodCollection` struct represents a collection of periods and has some useful methods on its own:

### `overlap_all(&[PeriodCollection]) -> Result<PeriodCollection, Error>`

Overlaps all collection periods on each other.

![Collection overlap all](/docs/period-rust/v1/img/collection-overlap-all.png)

### `subtract(&[Period]) -> Result<PeriodCollection, Error>`

Subtracts periods from a period collection. Pass `other_collection.as_ref()` to subtract a collection.

![Collection subtract](/docs/period-rust/v1/img/collection-subtract.png)

### `boundaries() -> Result<Option<Period>, Error>`

Creates a new period representing the outer boundaries of the collection.

![Collection boundaries](/docs/period-rust/v1/img/collection-boundaries.png)

### `gaps() -> Result<PeriodCollection, Error>`

Gives the gaps for all periods within this collection.

![Collection gaps](/docs/period-rust/v1/img/collection-gaps.png)

### `intersect(&Period) -> Result<PeriodCollection, Error>`

Intersects given period with every period within a collection. The result is a new collection of overlapping periods between given period and every period in the collection. When there's no overlap, the original period is discarded.

![Collection intersect](/docs/period-rust/v1/img/collection-intersect.png)

### `union() -> Result<PeriodCollection, Error>`

Merges all periods in collection with overlapping or touching ranges.

![Collection union](/docs/period-rust/v1/img/collection-union.png)

---

Finally, there are a few utility methods available on `PeriodCollection` as well:

- `add(periods)`: copies the collection with periods appended.
- `map(closure)`: maps each period into a new period.
- `reduce(initial, closure)`: folds the periods in input order.
- `filter(closure)`: keeps matching periods.
- `is_empty() -> bool`
- `len() -> usize`
- `sort()`: stable sort by included start.
- `unique()`: removes duplicate string representations, keeping the last value at its first position.

Collections support `FromIterator`, `Extend`, indexing and owned or borrowed iteration. Use `push`, indexed assignment or `remove` to change entries. The periods themselves remain immutable.

## Compatibility

You can construct a `Period` from Jiff dates or timezone-aware dates:

```rust
# use spatie_period::{Period, jiff::civil::date};
# fn main() -> Result<(), spatie_period::Error> {
Period::make(date(2021, 1, 1), date(2021, 1, 2))?;
# Ok(())
# }
```

Note that as soon as a period is constructed, all further operations on it are immutable.
There's never the danger of changing the input dates.

You can iterate a `Period` with the precision specified on creation:

```rust
# use spatie_period::{Period, Precision, Boundaries, jiff::civil::date};
# fn main() -> Result<(), spatie_period::Error> {
let date_period = Period::make(date(2021, 1, 1), date(2021, 1, 31))?;

for date in &date_period {
    let date = date?;
    // 2021-01-01
    // 2021-01-02
    // ...
    // (31 iterations)
    # let _ = date;
}

let time_period = Period::with_options(date(2021, 1, 1).at(0, 0, 0, 0), date(2021, 1, 1).at(23, 59, 59, 0), Precision::Hour, Boundaries::ExcludeNone)?;

for time in &time_period {
    let time = time?;
    // 2021-01-01 00:00:00
    // 2021-01-01 01:00:00
    // ...
    // (24 iterations)
    # let _ = time;
}
# assert_eq!(date_period.iter().count(), 31);
# assert_eq!(time_period.iter().count(), 24);
# Ok(())
# }
```

Date strings without a timezone and civil dates use UTC. Pass a `Zoned` to keep a named timezone, including daylight-saving transitions. RFC 3339 timestamps and Jiff's zoned strings are also accepted. Use `Period::with_format` for strftime-style custom formats such as `%d/%m/%Y`. Invalid dates return `Error` rather than being normalized into another month.

`Period::make` and `Period::with_options` round dates down. Use `Period::new` with `Zoned` values to preserve unrounded or fractional dates. Dates must fit Jiff's supported range, years -9999 through 9999, including any extra unit needed for boundary exclusions. Iterators yield `Result<Zoned, Error>` so arithmetic overflow can be handled without panicking.

## Visualizing periods

You can visualize one or more `Period` objects as well as `PeriodCollection`
objects to see how they related to one another:

```rust
# use spatie_period::{Period, PeriodCollection, Visualizer};
# fn main() -> Result<(), spatie_period::Error> {
let visualizer = Visualizer::new(27);

visualizer.visualize(&[
    ("A", PeriodCollection::new([Period::make("2021-01-01", "2021-01-31")?])),
    ("B", PeriodCollection::new([Period::make("2021-02-10", "2021-02-20")?])),
    ("C", PeriodCollection::new([Period::make("2021-03-01", "2021-03-31")?])),
    ("D", PeriodCollection::new([Period::make("2021-01-20", "2021-03-10")?])),
    ("OVERLAP", PeriodCollection::new([
        Period::make("2021-01-20", "2021-01-31")?,
        Period::make("2021-02-10", "2021-02-20")?,
        Period::make("2021-03-01", "2021-03-10")?,
    ])),
])?;
# Ok(())
# }
```

And visualize will return the following string:

```text
A          [========]
B                      [==]
C                           [========]
D               [==============]
OVERLAP         [===]  [==] [==]
```

The visualizer has a configurable width provided upon creation
which will control the bounds of the displayed periods:

```rust
# use spatie_period::Visualizer;
let visualizer = Visualizer::new(10);
```

The width controls bars only, not labels. Supply each row as a labeled `PeriodCollection`, wrapping a single period in a one-entry collection. An empty input returns an empty string; a zero-width visualizer returns labels only.

See [testing](testing.md) for the commands used to check the library and these examples.
