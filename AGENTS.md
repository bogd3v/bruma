# AGENTS.md

Instructions for coding agents working in this repository. Bruma is an air quality visualizer: Rust → WebAssembly, rendered with wgpu (WebGPU with a WebGL2 fallback) and built with Trunk. See `README.md` and `docs/decisions.md`.

## Engineering standard

The rules shared by every Micelio repository live in [`docs/engineering-standard.md`](docs/engineering-standard.md), the core, and in the Rust profile, [`docs/standard/rust.md`](docs/standard/rust.md). This repository follows standard version 2.3 with that profile (`standard: 2.3`, `profile: rust`; the `lint-standard` step of CI checks both copies against the hashes of `@micelio/eslint-config`). This file adds what is specific to this repository and never relaxes the standard. Bruma is not in `.github/standard-sync.json` of `micelio-engineering`, so no bot updates the copy: it moves to a new version by hand, byte for byte from `micelio-engineering`, together with `standard:` here and `lint-standard-version` in CI.

## Commands

```bash
install: cargo fetch --locked
test: cargo test --workspace --locked
check: scripts/check.sh
(cd web && trunk build --release)   # the production build, run by hand and by CI's own `web` job
(cd web && trunk serve)             # the development server
cargo build --workspace --locked --release
```

`scripts/check.sh` runs, in order: `cargo fmt --all --check`; `cargo clippy` for the workspace on the native target and on `wasm32-unknown-unknown`; `cargo test --workspace --locked`; `cargo doc` with warnings denied; `cargo deny check licenses`; `reuse lint`; and the copy of the standard (`micelio-lint-standard`, at the version CI pins as `lint-standard-version`). It needs `cargo-deny` (`cargo install cargo-deny --locked`), `reuse` and Node (`npx`). `cargo audit --deny warnings` (`cargo install cargo-audit --locked`) is run by CI.

## What stands in for Docker

Bruma has no image: its stack is the Rust toolchain of `rust-toolchain.toml` and Trunk. The run that stands in for a container is `scripts/check.sh`, then `(cd web && trunk build --release)` (Trunk v0.21.14, as CI installs it). The worker runs both in its worktree; the verifier runs both on a fresh clone of the change.

## Structure

- `crates/bruma-data/` — the data model and interpolation; knows nothing of the browser or the GPU.
- `crates/bruma-render/` — the wgpu renderer; knows nothing of the DOM or of cities.
- `web/` — the app: canvas, animation loop and minimal UI (Trunk, wasm-bindgen).
- `docs/decisions.md` — Bruma's technical decisions; new ones are appended, never deleted.
- `deny.toml` — the licence allowlist; `rust-toolchain.toml` — the pinned toolchain (and `wasm32-unknown-unknown`); `rust-version = "1.87"` in `Cargo.toml` is the lowest supported version, tested by its own CI job.
- `.github/workflows/ci.yml` — calls the reusable workflows of `micelio-engineering`; the Trunk build, the binary size and the `dist/` artifact are Bruma's own job.

Project skills (`.claude/skills/`): `rust-engineering` (workspace architecture, APIs, errors, tests, dependencies), `rust-wasm` (anything touching `web/`, the DOM, Trunk, binary size), `wgpu-webgpu` (anything touching `crates/bruma-render` and WGSL shaders).

## Rules

- Hard limits: `.claude/TEAM.md`. Never push, merge, tag, release or deploy without the human's approval, and never read secrets.
- Work moves through the gates of the Micelio factory ([`docs/factory.md`](https://github.com/bogd3v/micelio-engineering/blob/main/docs/factory.md) in `micelio-engineering`): planner, master, worker, blind reviewer, verifier, then the maintainer.
- **English only.** Everything written into this repository is in English: code identifiers, comments, doc comments, documentation, UI text, log and error messages, test names, CI step names, commit messages and PR titles and descriptions. Official proper names of data sources stay as they are (RMCAB, Secretaría Distrital de Ambiente, IDECA). Conversation with the maintainer may be in Spanish; what lands in the repo is not.
- Dependencies point one way: `web` → `bruma-render` / `bruma-data`. `bruma-data` knows nothing about the browser or the GPU; `bruma-render` knows nothing about the DOM or cities.
- Nothing depends on a specific city: Bogotá is configuration, not code.
- Everything drawn works within WebGL2 limits; WebGPU-only features go on a separate path with a fallback.
- MSRV 1.87, edition 2024.
- No `unwrap()` in production code; in release a panic kills the app (`panic = "abort"`).
- New decisions are appended to `docs/decisions.md` without deleting earlier ones.
- Commits: imperative mood, English, short subject line (72 characters at most), a body that explains the why when it is not obvious, signed off (`git commit -s`).
