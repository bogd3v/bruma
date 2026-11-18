#!/usr/bin/env bash
# The check command of AGENTS.md: the checks of section 15 of docs/standard/rust.md, in CI's order.
# Run it from anywhere; it stops at the first failure.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --target wasm32-unknown-unknown --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo deny check licenses
reuse lint
# The version is the one ci.yml pins as `lint-standard-version`.
npx --yes --package @micelio/eslint-config@0.2.10 micelio-lint-standard
