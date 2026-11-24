//! Period contracts, including calendar boundaries and timezone transitions.

use spatie_period::jiff::{ToSpan, Zoned, civil::date};
use spatie_period::{Boundaries, Error, IntoDate, Period, PeriodCollection, Precision, Visualizer};
use std::{cmp::Ordering, collections::BTreeSet};

fn period(value: &str) -> Period {
    value.parse().unwrap()
}

fn strings(periods: &PeriodCollection) -> Vec<String> {
    periods.iter().map(ToString::to_string).collect()
}

#[test]
fn string_notation_round_trips_every_precision_and_boundary() {
    for (start, end) in [
        ("2021", "2023"),
        ("2021-01", "2021-05"),
        ("2021-01-01", "2021-01-09"),
        ("2021-01-01 01", "2021-01-01 08"),
        ("2021-01-01 01:02", "2021-01-01 01:09"),
        ("2021-01-01 01:02:03", "2021-01-01 01:02:09"),
    ] {
        for (left, right) in [('(', ')'), ('(', ']'), ('[', ')'), ('[', ']')] {
            let value = format!("{left}{start},{end}{right}");
            assert_eq!(period(&value).to_string(), value);
        }
    }
    assert_eq!(
        period("( 2021-01-01 , 2021-01-09 ]").to_string(),
        "(2021-01-01,2021-01-09]"
    );
}

#[test]
fn invalid_inputs_and_incompatible_precisions_are_errors() {
    for value in [
        "",
        "[",
        "💥",
        "[2021,2021-02]",
        "[2021-02-29,2021-03-01]",
        "2021,2023",
        "[2022,2021]",
    ] {
        assert!(value.parse::<Period>().is_err(), "{value}");
    }
    let day = Period::make("2021-01-01", "2021-01-03").unwrap();
    let hour = period("[2021-01-01 00,2021-01-03 00]");
    assert!(matches!(
        day.overlaps_with(&hour),
        Err(Error::PrecisionMismatch)
    ));
    assert!(matches!(day.equals(&hour), Err(Error::PrecisionMismatch)));
    assert!(matches!(
        day.subtract(std::slice::from_ref(&hour)),
        Err(Error::PrecisionMismatch)
    ));
    assert!(matches!(
        day.overlap(std::slice::from_ref(&hour)),
        Err(Error::PrecisionMismatch)
    ));
    assert!(matches!(day.gap(&hour), Err(Error::PrecisionMismatch)));
    assert!(matches!(
        day.ceiling_end(Some(Precision::Second)),
        Err(Error::InvalidCeiling { .. })
    ));
    assert!(day.contains_period(&hour));
}

#[test]
fn date_comparisons_round_at_the_period_precision_and_use_included_endpoints() {
    let p = period("(2021-01-01,2021-01-05)");
    for (input, before_start, at_start, after_start, before_end, at_end, after_end) in [
        (
            "2021-01-01 23:59:59",
            false,
            false,
            true,
            false,
            false,
            true,
        ),
        (
            "2021-01-02 12:34:56",
            false,
            true,
            false,
            false,
            false,
            true,
        ),
        (
            "2021-01-03 01:02:03",
            true,
            false,
            false,
            false,
            false,
            true,
        ),
        (
            "2021-01-04 23:59:59",
            true,
            false,
            false,
            false,
            true,
            false,
        ),
        (
            "2021-01-05 00:00:00",
            true,
            false,
            false,
            true,
            false,
            false,
        ),
    ] {
        assert_eq!(p.starts_before(input).unwrap(), before_start);
        assert_eq!(p.starts_at(input).unwrap(), at_start);
        assert_eq!(p.starts_after(input).unwrap(), after_start);
        assert_eq!(
            p.starts_before_or_at(input).unwrap(),
            before_start || at_start
        );
        assert_eq!(
            p.starts_after_or_at(input).unwrap(),
            after_start || at_start
        );
        assert_eq!(p.ends_before(input).unwrap(), before_end);
        assert_eq!(p.ends_at(input).unwrap(), at_end);
        assert_eq!(p.ends_after(input).unwrap(), after_end);
        assert_eq!(p.ends_before_or_at(input).unwrap(), before_end || at_end);
        assert_eq!(p.ends_after_or_at(input).unwrap(), after_end || at_end);
    }
}

#[test]
fn set_operations_match_an_independent_day_set_for_all_boundary_combinations() {
    let base = date(2024, 1, 1);
    let days = |p: &Period| -> BTreeSet<_> { p.iter().map(|d| d.unwrap().date()).collect() };
    let collection_days =
        |c: PeriodCollection| -> BTreeSet<_> { c.iter().flat_map(&days).collect() };
    for start in 0..5 {
        for end in start + 2..7 {
            for left in Boundaries::ALL {
                for right in Boundaries::ALL {
                    let a = Period::with_options(
                        base,
                        base.checked_add(4.days()).unwrap(),
                        Precision::Day,
                        left,
                    )
                    .unwrap();
                    let b = Period::with_options(
                        base.checked_add(start.days()).unwrap(),
                        base.checked_add(end.days()).unwrap(),
                        Precision::Day,
                        right,
                    )
                    .unwrap();
                    let a_days = days(&a);
                    let b_days = days(&b);
                    assert_eq!(
                        collection_days(a.subtract(std::slice::from_ref(&b)).unwrap()),
                        a_days.difference(&b_days).copied().collect()
                    );
                    assert_eq!(
                        collection_days(a.diff_symmetric(&b).unwrap()),
                        a_days.symmetric_difference(&b_days).copied().collect()
                    );
                    let overlap: BTreeSet<_> = a
                        .overlap(std::slice::from_ref(&b))
                        .unwrap()
                        .as_ref()
                        .map(&days)
                        .unwrap_or_default();
                    assert_eq!(overlap, a_days.intersection(&b_days).copied().collect());
                    assert_eq!(a.overlaps_with(&b).unwrap(), !a_days.is_disjoint(&b_days));
                }
            }
        }
    }
}

#[test]
fn operations_keep_boundary_style_and_handle_no_operands() {
    let p = period("(2021-01-01,2021-02-01)");
    let other = period("[2021-01-10,2021-01-15]");
    assert_eq!(
        p.overlap(std::slice::from_ref(&other))
            .unwrap()
            .unwrap()
            .to_string(),
        "(2021-01-09,2021-01-16)"
    );
    assert_eq!(
        strings(&p.subtract(&[other]).unwrap()),
        ["(2021-01-01,2021-01-10)", "(2021-01-15,2021-02-01)"]
    );
    assert_eq!(p.subtract(&[]).unwrap(), PeriodCollection::new([p.clone()]));
    assert!(p.overlap(&[]).unwrap().is_none());
    assert!(p.overlap_any(&[]).unwrap().is_empty());
    assert_eq!(p.overlap(&[p.clone(), p.clone()]).unwrap().unwrap(), p);
}

#[test]
fn gap_is_symmetric_and_absent_for_touching_or_overlapping_periods() {
    let a = period("[2021-01-01,2021-01-05]");
    let b = period("[2021-01-09,2021-01-15]");
    assert_eq!(
        a.gap(&b).unwrap().unwrap().to_string(),
        "[2021-01-06,2021-01-08]"
    );
    assert_eq!(a.gap(&b).unwrap(), b.gap(&a).unwrap());
    assert!(a.gap(&period("[2021-01-06,2021-01-10]")).unwrap().is_none());
    assert!(a.gap(&period("[2021-01-05,2021-01-10]")).unwrap().is_none());
}

#[test]
fn empty_and_fractional_period_lengths_follow_php_iteration_rules() {
    for precision in Precision::ALL {
        for boundaries in [
            Boundaries::ExcludeStart,
            Boundaries::ExcludeEnd,
            Boundaries::ExcludeAll,
        ] {
            let empty =
                Period::with_options("2024-01-01", "2024-01-01", precision, boundaries).unwrap();
            assert_eq!(empty.length().unwrap(), 0, "{precision:?} {boundaries:?}");
            assert_eq!(empty.iter().count(), 0);
        }
        let start = "2024-01-01 00:00:00.750000".into_date().unwrap();
        let end = start
            .checked_add(precision.interval())
            .unwrap()
            .checked_sub(500.milliseconds())
            .unwrap();
        let p = Period::new(start, end, precision, Boundaries::ExcludeStart).unwrap();
        assert_eq!(p.length().unwrap(), 1, "{precision}");
        assert_eq!(p.iter().count(), 1, "{precision}");
    }
    let p = Period::new(
        "2024-01-01 23:00:00".into_date().unwrap(),
        "2024-01-02 01:00:00".into_date().unwrap(),
        Precision::Day,
        Boundaries::ExcludeNone,
    )
    .unwrap();
    assert_eq!(p.length().unwrap(), 1);
}

#[test]
fn date_formats_and_calendar_ceilings_are_supported() {
    let offset = Period::make("2021-01-01T01:00:00+05:30", "2021-01-02T01:00:00+05:30").unwrap();
    assert_eq!(offset.to_string(), "[2021-01-01,2021-01-02]");
    assert_eq!(offset.start().offset().seconds(), 19800);
    let formatted = Period::with_format(
        "01/01/2021 01:00 +0530",
        "01/02/2021 01:00 +0530",
        Precision::Day,
        Boundaries::ExcludeNone,
        "%m/%d/%Y %H:%M %z",
    )
    .unwrap();
    assert_eq!(formatted.start(), offset.start());
    assert_eq!(formatted.end(), offset.end());
    let p = Period::with_format(
        "01/01/2021",
        "01/31/2021",
        Precision::Day,
        Boundaries::ExcludeNone,
        "%m/%d/%Y",
    )
    .unwrap();
    assert_eq!(p.length().unwrap(), 31);
    assert_eq!(
        p.ceiling_end(None).unwrap().datetime().to_string(),
        "2021-01-31T23:59:59"
    );
    assert_eq!(
        p.ceiling_end(Some(Precision::Month))
            .unwrap()
            .datetime()
            .to_string(),
        "2021-01-31T23:59:59"
    );
    let p = period("[2020-02-01,2020-02-15]");
    assert_eq!(p.ceiling_end(Some(Precision::Month)).unwrap().day(), 29);
}

#[test]
fn midnight_transitions_preserve_calendar_days_and_boundary_styles() {
    for (zone, day) in [
        ("America/Havana", date(2020, 3, 8)),
        ("Asia/Beirut", date(2020, 3, 29)),
        ("America/Santiago", date(2020, 9, 6)),
        ("Atlantic/Azores", date(2020, 3, 29)),
        ("Asia/Tehran", date(2022, 3, 22)),
        ("America/Sao_Paulo", date(2018, 11, 4)),
        ("America/Asuncion", date(2020, 10, 4)),
        ("Asia/Gaza", date(2020, 3, 28)),
        ("Asia/Amman", date(2020, 3, 27)),
        ("America/Havana", date(2020, 11, 1)),
        ("Atlantic/Azores", date(2020, 10, 25)),
        ("Asia/Gaza", date(2020, 10, 24)),
        ("Asia/Amman", date(2020, 10, 30)),
    ] {
        let p = |from: i64, to: i64, bounds| {
            Period::with_options(
                day.checked_add(from.days())
                    .unwrap()
                    .at(12, 0, 0, 0)
                    .in_tz(zone)
                    .unwrap(),
                day.checked_add(to.days())
                    .unwrap()
                    .at(12, 0, 0, 0)
                    .in_tz(zone)
                    .unwrap(),
                Precision::Day,
                bounds,
            )
            .unwrap()
        };
        let before = p(-2, 0, Boundaries::ExcludeEnd);
        let after = p(0, 2, Boundaries::ExcludeStart);
        assert_eq!(before.length().unwrap(), 2, "{zone}");
        assert_eq!(after.length().unwrap(), 2, "{zone}");
        assert_eq!(
            before.gap(&after).unwrap().unwrap(),
            p(0, 0, Boundaries::ExcludeNone),
            "{zone}"
        );
        assert_eq!(
            after.renew().unwrap(),
            p(4, 4, Boundaries::ExcludeNone),
            "{zone}"
        );
        let whole = p(-3, 3, Boundaries::ExcludeNone);
        let remaining = whole.subtract(&[p(0, 0, Boundaries::ExcludeNone)]).unwrap();
        assert_eq!(remaining[0], p(-3, -1, Boundaries::ExcludeNone), "{zone}");
        assert_eq!(remaining[1], p(1, 3, Boundaries::ExcludeNone), "{zone}");
        assert_eq!(
            p(0, 1, Boundaries::ExcludeNone)
                .duration()
                .compare_to(p(10, 11, Boundaries::ExcludeNone).duration())
                .unwrap(),
            Ordering::Equal,
            "{zone}"
        );
    }
}

#[test]
fn repeated_hours_remain_distinct_for_rounding_and_operations() {
    let first: Zoned = "2020-10-25T02:15:30+02:00[Europe/Rome]".parse().unwrap();
    let second: Zoned = "2020-10-25T02:15:30+01:00[Europe/Rome]".parse().unwrap();
    for precision in [Precision::Hour, Precision::Minute, Precision::Second] {
        assert_eq!(
            precision.round_date(&first).unwrap().offset(),
            first.offset()
        );
        assert_eq!(
            precision.round_date(&second).unwrap().offset(),
            second.offset()
        );
        assert_eq!(
            precision.ceil_date(&first).unwrap().offset(),
            first.offset()
        );
        assert_eq!(
            precision.ceil_date(&second).unwrap().offset(),
            second.offset()
        );
    }
    let a = Period::with_options(&first, &first, Precision::Hour, Boundaries::ExcludeNone).unwrap();
    let b =
        Period::with_options(&second, &second, Precision::Hour, Boundaries::ExcludeNone).unwrap();
    assert!(!a.equals(&b).unwrap());
    assert!(a.touches_with(&b).unwrap());
    assert_eq!(a.renew().unwrap(), b);
    let whole = Period::with_options(
        "2020-10-25T01:00:00+02:00[Europe/Rome]",
        "2020-10-25T03:00:00+01:00[Europe/Rome]",
        Precision::Hour,
        Boundaries::ExcludeNone,
    )
    .unwrap();
    assert_eq!(whole.length().unwrap(), 4);
    assert_eq!(whole.iter().count(), 3);
    assert_eq!(whole.subtract(&[a, b]).unwrap().len(), 2);
}

#[test]
fn collections_preserve_order_duplicates_and_immutable_operations() {
    let a = period("[2021-01-01,2021-01-10]");
    let b = period("[2021-01-06,2021-01-15]");
    let c = period("[2021-01-20,2021-01-25]");
    let collection = PeriodCollection::new([c.clone(), a.clone(), b.clone(), a.clone()]);
    assert_eq!(
        strings(&collection.union().unwrap()),
        ["[2021-01-01,2021-01-15]", "[2021-01-20,2021-01-25]"]
    );
    assert_eq!(
        strings(&collection.gaps().unwrap()),
        ["[2021-01-16,2021-01-19]"]
    );
    assert_eq!(
        collection.unique(),
        PeriodCollection::new([c.clone(), a.clone(), b.clone()])
    );
    assert_eq!(
        collection.sort(),
        PeriodCollection::new([a.clone(), a.clone(), b.clone(), c.clone()])
    );
    assert_eq!(
        collection
            .overlap_all(&[PeriodCollection::new([b])])
            .unwrap()
            .len(),
        3
    );
    assert_eq!(collection.intersect(&a).unwrap().len(), 3);
    assert_eq!(collection.filter(|p| p.start() == a.start()).len(), 2);
    assert_eq!(collection.reduce(0, |total, _| total + 1), 4);
    assert_eq!(collection.map(Clone::clone), collection);
    assert_eq!(collection.add([c]).len(), 5);
    assert_eq!(collection.len(), 4);
    assert_eq!(collection.overlap_all(&[]).unwrap(), collection);
}

#[test]
fn empty_collections_and_indexed_mutation_use_rust_collection_conventions() {
    let mut collection = PeriodCollection::default();
    assert!(collection.boundaries().unwrap().is_none());
    assert!(collection.gaps().unwrap().is_empty());
    assert!(collection.union().unwrap().is_empty());
    let p = period("[2021,2022]");
    collection.extend([p.clone(), p.clone()]);
    assert_eq!(collection.remove(0), Some(p.clone()));
    collection[0] = period("[2023,2024]");
    assert_eq!(collection.get(0).unwrap().to_string(), "[2023,2024]");
    assert!(collection.remove(10).is_none());
    assert!(collection.get(10).is_none());
}

#[test]
fn duration_equality_agrees_with_calendar_ordering() {
    let a = period("[2021-01-01 00,2021-01-01 01]");
    let b = period("[2021-01-01 00,2021-01-01 12]");
    assert!(!a.duration().equals(b.duration()).unwrap());
    assert!(!b.duration().equals(a.duration()).unwrap());
    let same_duration = period("[2021-01-03 04,2021-01-03 05]");
    assert!(a.duration().equals(same_duration.duration()).unwrap());
    let excluded_start = period("(2021-01-01 00,2021-01-01 01]");
    assert!(!a.duration().equals(excluded_start.duration()).unwrap());
    assert_eq!(
        a.duration().compare_to(b.duration()).unwrap(),
        Ordering::Less
    );
    assert!(a.duration().is_smaller_than(b.duration()).unwrap());
    assert!(b.duration().is_larger_than(a.duration()).unwrap());
}

#[test]
fn duration_normalizes_a_multi_hour_fold_and_truncates_fractional_comparisons() {
    let p = Period::with_options(
        "2020-10-25T01:00:00+02:00[Antarctica/Troll]",
        "2020-10-25T02:00:00+00:00[Antarctica/Troll]",
        Precision::Hour,
        Boundaries::ExcludeAll,
    )
    .unwrap();
    assert_eq!(
        p.renew().unwrap().to_string(),
        "(2020-10-25 02,2020-10-25 04)"
    );
    let a = Period::new(
        "2024-01-01T00:00:00Z".into_date().unwrap(),
        "2024-01-01T00:00:00.5Z".into_date().unwrap(),
        Precision::Second,
        Boundaries::ExcludeNone,
    )
    .unwrap();
    let b = Period::make("2024-01-01", "2024-01-01").unwrap();
    assert_eq!(
        a.duration().compare_to(b.duration()).unwrap(),
        Ordering::Equal
    );
}

#[test]
fn mixed_timezone_endpoints_keep_calendar_duration_and_the_other_end_on_ties() {
    let a = Period::with_options(
        "2020-02-01T00:00:00+01:00[Europe/Rome]",
        "2020-02-01T00:00:00-05:00[America/New_York]",
        Precision::Hour,
        Boundaries::ExcludeNone,
    )
    .unwrap();
    let b = Period::with_options(
        "2020-02-01T01:00:00+01:00[Europe/Rome]",
        "2020-02-01T06:00:00+01:00[Europe/Rome]",
        Precision::Hour,
        Boundaries::ExcludeNone,
    )
    .unwrap();
    assert_eq!(
        a.overlap(std::slice::from_ref(&b))
            .unwrap()
            .unwrap()
            .to_string(),
        "[2020-02-01 01,2020-02-01 06]"
    );
    assert_eq!(
        a.renew().unwrap().to_string(),
        "[2020-02-01 01,2020-02-01 07]"
    );
    assert!(a.duration().is_larger_than(b.duration()).unwrap());
}

#[test]
fn repeated_empty_subtractions_keep_php_fragment_multiplicity() {
    let whole = Period::make(
        "2020-03-06T12:00:00-05:00[America/Havana]",
        "2020-03-10T12:00:00-04:00[America/Havana]",
    )
    .unwrap();
    let empty = Period::with_options(
        "2020-03-08T12:00:00-04:00[America/Havana]",
        "2020-03-08T12:00:00-04:00[America/Havana]",
        Precision::Day,
        Boundaries::ExcludeAll,
    )
    .unwrap();
    assert_eq!(
        strings(&whole.subtract(&[empty.clone(), empty]).unwrap()),
        [
            "[2020-03-06,2020-03-08]",
            "[2020-03-08,2020-03-08]",
            "[2020-03-08,2020-03-08]",
            "[2020-03-08,2020-03-10]"
        ]
    );
}

#[test]
fn visualizer_matches_php_output_for_collections_and_single_units() {
    let output = Visualizer::new(20)
        .visualize(&[
            (
                "A",
                PeriodCollection::new([period("[2018-01-01,2018-01-20]")]),
            ),
            (
                "B",
                PeriodCollection::new([
                    period("[2018-01-01,2018-01-05]"),
                    period("[2018-01-11,2018-01-15]"),
                ]),
            ),
            (
                "C",
                PeriodCollection::new([period("[2018-01-10,2018-01-10]")]),
            ),
        ])
        .unwrap();
    assert_eq!(
        output,
        "A    [==================]\nB    [===]     [===]     \nC             =          "
    );
}
