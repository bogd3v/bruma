---
name: rust-engineering
description: Rust engineering practices for Bruma — workspace crate boundaries, API design, error handling, testing, lints, dependencies and the decision log. Use when creating or refactoring any module, public type or crate, when adding dependencies, or when reviewing Rust code in this project.
---

# Rust engineering — Bruma

## Language

Everything in the repository is in English: identifiers, comments, doc comments, error and
log messages, UI text, test names, docs and commit messages. Official data-source names
(RMCAB, IDECA, Secretaría Distrital de Ambiente) keep their original spelling.

## Workspace architecture

Dependencies flow in one direction only:

```
web (bruma-web)  ──►  bruma-render  ──►  wgpu
       │
       └────────────►  bruma-data   (no browser or GPU dependencies)
```

- **`bruma-data`**: model, cleaning, source adapters, interpolation (IDW). Pure Rust: no
  `web-sys`, no `wgpu`, no `wasm-bindgen`. Tested natively with `cargo test`.
- **`bruma-render`**: everything that touches the GPU. Knows nothing about the DOM: it
  receives an already created `wgpu::Surface`. Knows nothing about cities or sources: it
  receives GPU-ready data (`f32`, grids).
- **`web`**: the glue with the browser (canvas, `requestAnimationFrame`, events, fetch, UI
  text). As thin as possible; any testable logic moves down into a library crate.
- **Nothing depends on a specific city.** Bogotá is configuration and test data, never an `if`.
- Before adding a crate, ask whether a module is enough. A new crate is justified by a
  dependency boundary (e.g. the future data proxy), not by size.

## Types and API design

- **Make invalid states unrepresentable**: enums instead of magic `bool`/`String`, newtypes
  for units (`Pm25(f64)`, `Lon(f64)`) when mixing them up is a real risk.
- **Validate at the boundary, trust inside**: raw network data is cleaned once
  (`clean_reading`) and the rest of the code works with valid values.
- Minimal public surface: `pub` only for what another crate uses; otherwise `pub(crate)` or
  private.
- Borrow in arguments (`&str`, `&[T]`), return owned values.
- Fallible constructors return `Result`; no `new()` that panics.
- Prefer pure functions (input → output) for data logic: trivial to test and easy to move
  to the GPU later.
- `#[derive(Debug)]` on every public type; `Clone, Copy, PartialEq, Eq, Hash` when they make
  semantic sense, not by habit.

## Errors

- Library crates (`bruma-data`, `bruma-render`) define **one error enum per crate** (or per
  large module) implementing `Display` + `std::error::Error`, like `RenderError`. If the enum
  grows, `thiserror` is fine; no `anyhow` in libraries.
- `web` may reduce errors to a user-facing message, but it must say what happened and what
  the person can do.
- **No `unwrap()`/`expect()` in production code** except for provable invariants, and then
  `expect("why this cannot fail")`. Fine in tests.
- Release uses `panic = "abort"`: a panic in Wasm kills the whole app. Treat every possible
  panic as a bug.
- Never swallow errors silently: propagate with `?`, or log with `log::warn!` and explain in a
  comment why continuing is safe (like the skipped frame in `render`).

## Numbers and units

- `f64` for data and geography on the CPU; `f32` only when crossing to the GPU, at a single
  conversion point.
- Coordinates: project lon/lat onto a local plane in `f64` and send the GPU positions
  **relative to the city origin** in `f32` (avoids precision loss with large numbers).
- Document each numeric field's unit in its doc comment (µg/m³, m/s, degrees).
- Compare `f64` with `==` only in tests with exact values; otherwise use a tolerance.

## Style

- Edition 2024, **MSRV 1.87** (workspace `rust-version`). Do not use newer features: for
  example, chained `if let ... && ...` (let chains) requires 1.88.
- Comments explain the *why*, not the *what*.
- A `//!` header in every crate/module stating its responsibility and what it does *not* do.
- Short functions with a single responsibility. If a function needs section comments, it is
  probably several functions.
- No global mutable state (`static mut`, `lazy_static` with a `Mutex`) without a documented
  reason.
- `unsafe` is forbidden unless justified in a `// SAFETY:` comment and explicitly approved.
  `bytemuck` covers the byte-casting cases for the GPU.

## Dependencies

- Every dependency is declared in `[workspace.dependencies]` and inherited by crates with
  `.workspace = true`.
- `default-features = false`, enabling only the features needed (Wasm size matters).
- Before adding one: how much does it weigh in Wasm? Does it compile for
  `wasm32-unknown-unknown`? Is it maintained? Is its license compatible with MIT/Apache-2.0?
- Record any dependency that changes the architecture in `docs/decisions.md`.

## Testing

- Unit tests in `#[cfg(test)] mod tests` next to the code; names that describe behavior
  (`drops_no_data_codes`, not `test1`).
- Every cleaning rule or formula (IDW, projection, color scales) gets edge-case tests: empty
  input, a single point, NaN, values at the range limits, zero distance.
- For numeric logic consider property tests (`proptest`) in `dev-dependencies`.
- Small, hand-written test data in the repo; never download data in tests (the repo does not
  store copies of real data).
- Every bug fix comes with a test that reproduces it.

## Decision log

`docs/decisions.md` is the project's ADR log. When a decision changes (backend, data format,
build tool, GPU limits), append the new one with its reasoning and do **not** delete the old one.

## Commits

English, imperative mood, subject ≤ 72 characters, body explaining the why when it is not
obvious (`Add IDW interpolation to bruma-data`, not `Added stuff`).

## Verification before calling something done

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace
(cd web && trunk build --release)   # if web/ or bruma-render changed
```

This is what CI runs (`.github/workflows/ci.yml`). If something fails, report it with the
output; do not mark it as done.
