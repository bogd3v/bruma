---
name: rust-wasm
description: Rust compiled to WebAssembly for the browser in Bruma — Trunk, wasm-bindgen, web-sys, closures and requestAnimationFrame, async with wasm-bindgen-futures, binary size, panics and wasm32 testing. Use when touching web/, adding DOM interaction or fetch, changing Trunk.toml or the release profile, or when something compiles natively but fails in the browser.
---

# Rust → WebAssembly — Bruma

All code, comments, UI text and messages are in English (see `CLAUDE.md`).

## Toolchain

- Target: `wasm32-unknown-unknown`. Build and dev server with **Trunk** from `web/`
  (`trunk serve --open`, `trunk build --release`). Trunk runs `wasm-bindgen` and `wasm-opt`.
- Trunk downloads the `wasm-bindgen` CLI version matching `Cargo.lock`. A "schema version
  mismatch" error means the crate and CLI versions differ.
- wasm-pack only comes in if `bruma-data` is published to npm (see `docs/decisions.md`).

## What does not exist on `wasm32-unknown-unknown`

These compile fine and **panic or fail in the browser**:

| Don't use | Use instead |
| --- | --- |
| `std::time::Instant` / `SystemTime::now()` | `performance.now()` via `web-sys`, or the `web-time` crate |
| `std::thread`, blocking `std::sync::mpsc` | `wasm_bindgen_futures::spawn_local`; Web Workers only if truly needed |
| `std::fs`, `std::env`, `std::process` | fetch, `localStorage`, config embedded with `include_str!` |
| Blocking (`block_on`, busy-wait loops) | `async` + `await`; the main thread never blocks |
| Unconfigured `getrandom` | `getrandom`'s `wasm_js` feature (plus the `cfg` its version requires) |
| `reqwest` with native features | `web_sys::window().fetch_with_request` or `gloo-net` |

Rule: if any of these shows up in `bruma-data` or `bruma-render`, it is in the wrong crate or
needs a `#[cfg(target_arch = "wasm32")]` with an alternative (like `create_surface` in
`web/src/main.rs`).

## JavaScript interop (wasm-bindgen / web-sys)

- **Minimal `web-sys` features**: each DOM API is a feature in `web/Cargo.toml`. Add only the
  ones you use and keep them alphabetically sorted.
- Convert with `dyn_into::<T>()` and handle the error; never `unchecked_into` on anything that
  comes from the DOM.
- `Result<_, JsValue>` values from the DOM are converted into an error with a message before
  propagating.
- Hand-written JavaScript stays at zero or near zero; if needed, `#[wasm_bindgen(inline_js)]`
  or a small module next to `index.html`, documented.

## Closures and the frame loop

- A `Closure<dyn FnMut()>` must live as long as JS may call it. Two valid patterns:
  1. Store it (like `FrameCallback = Rc<RefCell<Option<Closure<..>>>>` for
     `requestAnimationFrame`).
  2. `closure.forget()` **only** for listeners that live for the whole page (e.g. `resize`),
     with a comment saying so. Every `forget` is an intentional leak.
- State shared between callbacks: `Rc<RefCell<T>>` (Wasm is single-threaded, don't use
  `Arc<Mutex<T>>`). Never hold a `borrow_mut()` across an `await` or a call that could
  re-enter the same callback.
- In the frame callback: no large allocations or `format!` per frame; read the canvas size,
  update uniforms, draw.
- Respect `prefers-reduced-motion` in every new animation.

## Async

- Entry point: `main()` installs `console_error_panic_hook` and `console_log`, then launches
  `spawn_local(async { ... })`. Errors from `start()` are shown in `#bruma-status`.
- Data requests go to our own proxy (`/api`, same origin), never directly to the source from
  the browser.
- Every `fetch` handles: network error, non-2xx status, invalid body. The user sees a clear
  state (loading / no data / error) and the page never goes blank.

## Panics and errors

- Release uses `panic = "abort"`: a panic leaves the canvas frozen. `console_error_panic_hook`
  only helps debugging; the goal is zero panics.
- Index with `get()` instead of `[]` when the index comes from external data.
- Numeric values from the DOM (`client_width`, `device_pixel_ratio`) are clamped
  (`max(1)`, `clamp`) before being used as GPU sizes.

## Binary size

Initial download is the metric that matters most (profile `opt-level = "z"`, LTO, `strip`).

- CI reports the `.wasm` size (raw and gzip) on every build; compare it before and after
  adding a dependency or feature.
- To investigate: `twiggy top web/dist/*.wasm` or
  `cargo bloat --target wasm32-unknown-unknown --release`.
- Typical bloat: `serde_json` with many types, `regex`, `{:?}` formatting everywhere, generics
  monomorphized many times, dependencies' default features.
- Prefer compact data formats from the proxy (small flat JSON, or simple binary) if parsing
  starts to weigh.

## Testing

- Logic is tested natively (`cargo test`) because it lives in crates without browser
  dependencies. That is the main path.
- For browser-only code, `wasm-bindgen-test` in `dev-dependencies` with
  `wasm_bindgen_test_configure!(run_in_browser);`, run with
  `wasm-pack test --headless --firefox` (or `--chrome`). Use sparingly: it is slow.
- Before closing a change in `web/`, also build for wasm:
  `cargo clippy -p bruma-web -p bruma-render --target wasm32-unknown-unknown -- -D warnings`
  and `cd web && trunk build --release`.
- Manual check: `trunk serve`, open in a browser with WebGPU and in one without it (e.g.
  Firefox on Linux) to confirm both paths; the console must be free of errors and warnings.
