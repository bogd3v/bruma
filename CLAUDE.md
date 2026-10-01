# Bruma

Air quality visualizer: Rust → WebAssembly, rendering with wgpu (WebGPU with a WebGL2
fallback), built with Trunk. See `README.md` and `docs/decisions.md`.

## Language: English only

Everything written into this repository is in **English**: code identifiers, comments, doc
comments, documentation, UI text, log and error messages, test names, CI step names, commit
messages and PR titles/descriptions. Official proper names of data sources stay as they are
(RMCAB, Secretaría Distrital de Ambiente, IDECA). Conversation with the maintainer may be in
Spanish; what lands in the repo is not.

## Project skills (`.claude/skills/`)

- `rust-engineering` — workspace architecture, APIs, errors, tests, dependencies.
- `rust-wasm` — anything touching `web/`, the DOM, Trunk, binary size.
- `wgpu-webgpu` — anything touching `crates/bruma-render` and WGSL shaders.

## Rules that always apply

- Dependencies point one way: `web` → `bruma-render` / `bruma-data`. `bruma-data` knows
  nothing about the browser or the GPU; `bruma-render` knows nothing about the DOM or cities.
- Nothing depends on a specific city: Bogotá is configuration, not code.
- Everything drawn works within WebGL2 limits; WebGPU-only features go on a separate path
  with a fallback.
- MSRV 1.87, edition 2024.
- No `unwrap()` in production code; in release a panic kills the app (`panic = "abort"`).
- New decisions are appended to `docs/decisions.md` without deleting earlier ones.

## Commits

Imperative mood, English, short subject line (≤ 72 chars), body explaining the why when it
is not obvious. Example: `Add IDW interpolation to bruma-data`.

## Verification (same as CI)

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p bruma-web -p bruma-render --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace
(cd web && trunk build --release)
```
