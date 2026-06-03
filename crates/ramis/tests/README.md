# Tests

This directory contains integration tests for schedulers implemented in this repo. Tests may be run using shuttle or default.

## Running

To run tests:

```bash
  cargo test --locked --all-features
```

To run `shuttle` tests:

```bash
  RUSTFLAGS="--cfg shuttle" cargo test --release --locked -p ramis
```
