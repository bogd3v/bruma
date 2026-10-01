# Technical decisions

A short record of Bruma's decisions and why they were made. Any of them can change; when
one does, the new decision is added here instead of deleting the old one.

## Language and build

- **The whole core in Rust**, compiled to WebAssembly. JavaScript is kept to the minimum
  that wasm-bindgen generates.
- **Trunk** as the build tool: Bruma is a web application, not an npm library. Trunk starts
  from `web/index.html`, compiles to Wasm, generates the wasm-bindgen glue and runs
  `wasm-opt`. wasm-pack would only come in if `bruma-data` is ever published as an npm
  package.
- **Release profile** tuned for size (`opt-level = "z"`, LTO, `panic = "abort"`): on the web,
  the initial download weighs more than a few milliseconds of CPU.

## Graphics

- **wgpu** as the only rendering API: it talks WebGPU when the browser supports it and
  WebGL2 when it does not.
- **Detection before touching the canvas:** a `<canvas>` only accepts one context type, so
  WebGPU availability is checked first (`new_instance_with_webgpu_detection`) and then the
  surface is created.
- **WebGL2 limits for both backends:** the device requests the limits WebGL2 guarantees, so
  the same code behaves the same on both. WebGPU-only features (compute shaders) arrive in
  Phase 2 on their own path.
- **Shaders validated in `cargo test`** with naga.
- **Accessibility:** with `prefers-reduced-motion` the animation stops.

## Cities

- **Bruma works for any city.** Nothing in the renderer or the data model depends on
  Bogotá. A city is a configuration: name, map area, outline, data sources and attribution.
- **Bogotá is the first city** and the reference case everything is tested against.
- **Each monitoring network is an adapter** that translates its data into the common
  `bruma-data` model. Adding a city with an already supported source requires no code.

## Data

- **Bogotá:** the public hourly report from RMCAB (Secretaría Distrital de Ambiente). OpenAQ
  does not work for Bogotá in real time: its data for the city stopped updating in 2022 and
  does not include wind.
- **Generic source:** OpenAQ (API v3) for cities where it has recent data. Its API key lives
  only in the proxy.
- **Always through the proxy:** each source is queried once per hour from our own server,
  never from each visitor's browser.
- **Wind fallback:** Open-Meteo (modeled data, labeled as such), available for any
  coordinate.
- **Cartography:** OpenStreetMap (ODbL) as the base for any city; open official sources when
  they exist, such as IDECA – Mapa de Referencia de Bogotá (CC BY 4.0).
- **Cleaning:** "no data" codes (`-9999`) and values outside a plausible per-variable range
  are discarded. The ranges are provisional until we have the network's official
  documentation.

## Infrastructure

- **Dokploy** on the same VPS as the blog, at no extra cost.
- **A single subdomain**, `bruma.bogdev.com.co`: the site at `/` and the data proxy at
  `/api`, routed by Traefik (same origin, no CORS).
- The DNS record is declared in BogDev's infrastructure repository (Terraform).

## Repository conventions

- **English everywhere** (decided 2026-10-01): code, comments, documentation, UI text, log
  and error messages, CI step names and commit messages. Earlier commits stay as they are;
  history is not rewritten. Data source names keep their official names (e.g. RMCAB,
  Secretaría Distrital de Ambiente).
