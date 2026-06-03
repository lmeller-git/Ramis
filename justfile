default:
    @just --list

build-py:
    uv sync
    uv run maturin develop

test-rust:
        cargo test --exclude lib-ramis --workspace --locked --all-features --all-targets
        cargo test --exclude lib-ramis --workspace --locked --all-features --doc
        cargo test -p lib-ramis --no-default-features --locked --all-targets
        cargo test -p lib-ramis --no-default-features --locked --doc

test-py: build-py
    uv run pytest

test: test-rust test-py

lint:
    cargo +nightly fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    uv run ruff check python
    uv run ruff format --check python
    uv run pyright

docs:
    cargo +nightly docs-rs -p ramis
    cargo +nightly docs-rs -p lib-ramis

hack:
    cargo hack --feature-powerset check

check: lint docs hack
