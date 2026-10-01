---
name: wgpu-webgpu
description: GPU programming in Bruma with wgpu and WGSL — WebGPU with WebGL2 fallback, limits, uniform layout and bytemuck, pipelines and resources, compute shaders (Phase 2), per-frame performance, shader validation with naga, and debugging. Use when touching crates/bruma-render, writing or changing .wgsl shaders, adding buffers, textures or pipelines, or planning GPU work (heat map, IDW, particles).
---

# GPU with wgpu and WGSL — Bruma

All code, shader comments, labels and messages are in English (see `CLAUDE.md`).

wgpu is pinned to the version in the workspace `Cargo.toml`. Its API changes between major
versions: before using a type or method not already present in `crates/bruma-render`, check
its signature on docs.rs for **that** version instead of trusting older examples.

## Two backends, one codebase

- `wgpu::Backends::BROWSER_WEBGPU | GL` with `new_instance_with_webgpu_detection`: detection
  happens before creating the surface because a `<canvas>` accepts only one context.
- The device is requested with
  `Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())`.
  **Everything drawn must work within those limits.**
- What WebGL2 lacks (assume it is missing on the common path):
  - compute shaders,
  - storage buffers and storage textures (`var<storage>`),
  - many bindings and large uniform buffers (respect `max_uniform_buffer_binding_size`),
  - textures larger than the device's `max_texture_dimension_2d` (the canvas is already clamped).
- WebGPU-only features (compute for IDW/particles in Phase 2) go on **a separate path**,
  chosen at runtime with
  `adapter.get_downlevel_capabilities().flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)`,
  with a CPU or fragment-shader equivalent for WebGL2. The visual result must be the same or
  degrade gracefully, never break.
- Record any change to requested limits or features in `docs/decisions.md`.

## Uniforms and data sent to the GPU

- Every struct crossing to the GPU is `#[repr(C)]` + `bytemuck::Pod + Zeroable` and has a
  comment naming its WGSL twin (like `Globals`).
- WGSL layout rules for `var<uniform>`:
  - struct size is rounded up to a multiple of 16 bytes;
  - `vec3<f32>` is 16-byte aligned (but 12 bytes long): **avoid `vec3` in uniforms**, use
    `vec4` or scalars + explicit padding;
  - arrays in uniforms have a 16-byte stride per element.
- Explicit `_pad` fields on both sides, plus a compile-time assertion:
  `const _: () = assert!(std::mem::size_of::<Globals>() % 16 == 0);`
- Better still, a test comparing `size_of::<T>()` with the size naga computes for the WGSL
  struct (`module.types` + `naga::proc::Layouter`), so a mismatch fails in `cargo test`.
- Per-vertex/instance data: `VertexBufferLayout` with `wgpu::vertex_attr_array!` and
  `array_stride = size_of::<T>()`.

## Resources and lifetime

- Pipelines, layouts, bind groups, buffers and textures are created **once** (in `new` or when
  the data changes), never per frame.
- Per frame only: `queue.write_buffer` for uniforms, one `CommandEncoder`, the passes,
  `submit` and `present`.
- Data that changes hourly (station readings): rewrite the existing buffer if it fits;
  recreate buffer + bind group only when the size changes.
- `label: Some("...")` on every resource: it is what appears in validation errors.
- Surface handling as in `Renderer::render`: `Suboptimal` → use it and reconfigure;
  `Outdated`/`Lost` → reconfigure and skip the frame; anything else → `log::warn!` and skip.
- Sizes always ≥ 1 and ≤ `max_texture_dimension_2d`.
- The surface format comes from `get_capabilities`; if the pipeline assumes sRGB or linear,
  say so in a comment and convert color where needed (the palette and air quality color
  scales must look the same on both backends).

## WGSL

- One `.wgsl` file per pipeline in `crates/bruma-render/src/shaders/`, loaded with
  `include_str!`. Entry points `vs_main` / `fs_main` / `cs_main`.
- Comments explain intent and units (clip space, UV, city coordinates).
- **Every new shader gets its own naga validation test** (like
  `triangle_shader_is_valid_wgsl`). To guarantee WebGL2 compatibility, the test can also
  translate it to GLSL ES 3.00 with naga's `glsl-out` backend: if it does not translate, it
  will not run on WebGL2.
- Avoid data-dependent branches in hot fragment shaders; prefer `select`, `mix`, `step`,
  `smoothstep`.
- Guard divisions against zero (IDW at distance 0 → use the station's value).
- Precision: `f32` on the GPU. Positions relative to the city origin, never raw lon/lat.

## Compute (Phase 2, WebGPU only)

- `@workgroup_size(64)` (or `8, 8` for 2D grids) as a starting point; dispatch with rounded-up
  division: `n.div_ceil(64)`, and in the shader
  `if (id.x >= arrayLength(&data)) { return; }`.
- Reading results back to the CPU (`map_async`) is async and expensive: avoid it in the frame
  loop. If the data is only drawn, keep it on the GPU (compute writes → render reads).
- The CPU vs GPU benchmark measures the same work on both sides, at several input sizes, and
  discards the first frame (pipeline compilation).

## Performance

- Measure before optimizing: frame time in the browser DevTools; timestamp queries only if the
  adapter offers them (not on WebGL2).
- Few draw calls: instancing for stations/particles instead of one draw per object.
- The heat map is computed at a fixed grid resolution independent of the canvas size, and
  scaled up with linear filtering.
- `PresentMode::Fifo` (vsync). When nothing animates (reduced motion, static data), consider
  drawing only when something changes instead of on every `requestAnimationFrame`.

## Debugging

- wgpu validation errors appear in the browser console with the resource `label`.
- In development, register handlers for uncaptured device errors and device loss
  (`Device::on_uncaptured_error`, `set_device_lost_callback`; confirm the signature for the
  pinned version) and show them in `#bruma-status` instead of failing silently.
- Black screen: check in order clear color → viewport/surface size → correct bind group →
  uniform layout (padding) → clip coordinates outside [-1, 1] → winding and culling.
- Always test both paths — a browser with WebGPU and one without (Firefox on Linux) — before
  calling a render change done.

## Verification

```bash
cargo test -p bruma-render                       # includes shader validation
cargo clippy -p bruma-render --target wasm32-unknown-unknown -- -D warnings
(cd web && trunk serve)                          # check on WebGPU and on WebGL2
```
