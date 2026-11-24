---
title: Introduction
weight: 1
---

This package adds support for comparing multiple dates with each other.
You can calculate the overlaps and differences between n-amount of periods,
as well as some more basic comparisons between two periods.

Periods can be constructed from date strings or [Jiff](https://docs.rs/jiff) dates,
timestamps and timezone-aware dates. Implement `IntoDate` to accept another date type.

Periods are always immutable, there's never the worry about your input dates being changed.

Continue with [installation](installation-and-setup.md) and [usage](usage.md).
