default:
    @just --list

build-py:
    uv sync

test-rust:
    cargo test --workspace --exclude lib_ramis --all-targets

test-py: build-py
    uv run pytest

test: test-rust test-py

lint:
    cargo +nightly fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    uv run ruff check python
    uv run ruff format --check python
    uv run pyright
