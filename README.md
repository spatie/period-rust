# spatie-period

[![Latest version on crates.io](https://img.shields.io/crates/v/spatie-period.svg?style=flat-square)](https://crates.io/crates/spatie-period)
[![Tests](https://img.shields.io/github/actions/workflow/status/spatie/period-rust/ci.yml?branch=main&style=flat-square)](https://github.com/spatie/period-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/spatie/period-rust/blob/main/LICENSE.md)
[![Total downloads](https://img.shields.io/crates/d/spatie-period.svg?style=flat-square)](https://crates.io/crates/spatie-period)

This package adds support for comparing multiple dates with each other.
You can calculate the overlaps and differences between n-amount of periods,
as well as some more basic comparisons between two periods.

Periods are always immutable, there's never the worry about your input dates being changed.

## Support us

[<img src="https://spatie.be/github-ad-image/spatie-period.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/spatie-period)

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

Requires Rust 1.89 or newer. Once the crate is published:

```sh
cargo add spatie-period
```

Before publication, use a path dependency from a neighboring application:

```toml
[dependencies]
spatie-period = { path = "../period-rust" }
```

## Usage

```rust
# use spatie_period::Period;
# fn main() -> Result<(), spatie_period::Error> {
let period = Period::make("2021-01-01", "2021-01-31")?;

assert_eq!(period.length()?, 31);
# Ok(())
# }
```

See the [documentation](https://spatie.be/docs/period-rust/v1/introduction) for precision, boundaries, comparisons, operations, collections and visualization.

## Testing

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
```

See [all quality checks](https://github.com/spatie/period-rust/blob/main/docs/testing.md). The README and Markdown guides are tested by rustdoc, including setup on hidden `# ` lines.

## Changelog

See [CHANGELOG.md](https://github.com/spatie/period-rust/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/spatie/period-rust/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/spatie/period-rust/blob/main/SECURITY.md).

## Credits

- [Brent Roose](https://github.com/brendt), author of the [PHP package](https://github.com/spatie/period)
- [Freek Van der Herten](https://github.com/freekmurze)
- [All contributors](https://github.com/spatie/period-rust/graphs/contributors)

## License

The MIT License (MIT). See [LICENSE.md](https://github.com/spatie/period-rust/blob/main/LICENSE.md).
