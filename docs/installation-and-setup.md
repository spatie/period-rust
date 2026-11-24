---
title: Installation
weight: 3
---

Requires Rust 1.89 or newer. Install Rust using [rustup](https://rustup.rs/).

Once the crate is published, add it to your application:

```sh
cargo add spatie-period
```

Before publication, use a local path dependency:

```toml
[dependencies]
spatie-period = { path = "../period-rust" }
```
