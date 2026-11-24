# Changelog

All notable changes to `spatie-period` will be documented in this file.

## 2.0.0 - 2026-11-24

- Breaking: duration equality now agrees with calendar duration ordering and respects included endpoints. Equal whole-day counts no longer make different hour durations equal.
- Focus the guides on Rust date inputs and comparisons.

## 1.0.0 - 2026-11-24

- Port period precision, boundaries, comparisons, overlap, subtraction, gaps, symmetric difference, renewal and duration comparisons from spatie/period.
- Add ordered period collections, calendar iteration and ASCII visualization.
- Add executable v1 documentation and Rust 1.89, cross-platform and security checks.
