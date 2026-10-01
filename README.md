# Bruma

Your city's air quality, drawn with the GPU.

Bruma is a web visualizer that takes data from a city's monitoring stations, processes it
with **Rust compiled to WebAssembly** and draws it with **WebGPU** (with a WebGL2
fallback). It works for any city whose stations publish public data: each city is a
configuration and each monitoring network is an adapter. **Bogotá is the first city** and
the reference case.

It is an open project by [BogDev](https://bogdev.com.co), built in stages, each one told in
a series of blog posts.

> **Status: Phase 0.** The repository, the Wasm build and the first triangle on the GPU.
> There is no air quality data yet.

## What Phase 0 looks like

An animated triangle on a full-screen canvas. In the top right corner, Bruma shows which
graphics API the browser picked: **WebGPU** if available, **WebGL2** otherwise. The same
Rust code runs in both cases. If the system asks for reduced motion
(`prefers-reduced-motion`), the animation stays still.

## Structure

```
bruma/
├── crates/
│   ├── bruma-data/     # data model and cleaning, shared by every city (Phase 1: adapters and IDW)
│   └── bruma-render/   # wgpu + WGSL shaders
├── web/                # web app: index.html, styles and Rust entry point (Trunk)
├── docs/               # technical decisions
├── LICENSE-MIT
└── LICENSE-APACHE
```

## Requirements

- Stable Rust (1.87 or newer) with the WebAssembly target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [Trunk](https://trunkrs.dev) 0.21:
  ```bash
  cargo install --locked trunk
  ```

## Development

```bash
cd web
trunk serve --open        # http://127.0.0.1:8080, with live reload
```

Production build (output in `web/dist/`, ready to serve as a static site):

```bash
cd web
trunk build --release
```

Tests, formatting and lint (run natively, no browser needed):

```bash
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

The `bruma-render` test validates the WGSL shader with naga, so a shader syntax error shows
up in `cargo test` instead of when the page is opened.

## Compatibility

| Browser | API Bruma uses |
| --- | --- |
| Recent Chrome / Edge | WebGPU |
| Safari 26+ (macOS, iOS, iPadOS) | WebGPU |
| Recent Firefox (Windows, macOS) | WebGPU |
| Firefox on Linux and browsers without WebGPU | WebGL2 |

## Roadmap

- **Phase 0 — Foundations** (current): workspace, Trunk, CI and the first render on WebGPU and WebGL2.
- **Phase 1 — MVP:** city configuration, data adapters (RMCAB for Bogotá and OpenAQ as a generic source), IDW interpolation and heat map.
- **Phase 2 — v1:** compute-shader particles, wind, 24-hour timeline, more cities and a CPU vs GPU benchmark.

## Add your city

Starting in Phase 1 it will be enough to describe the city (name, map area, outline) and
choose where its data comes from:

- **OpenAQ**, if the city has stations with recent data there: no code needed.
- **A custom adapter**, if the local network publishes its data some other way (like RMCAB
  in Bogotá).

If you want to see your city in Bruma, open an issue with the city's name and where it
publishes its air quality data.

Design decisions are recorded in [`docs/decisions.md`](docs/decisions.md).

## Data and attribution

Starting in Phase 1, each city shows the attribution for its own sources. In every case,
values between stations are interpolated estimates, and Bruma does not replace official
alert channels.

- **Bogotá:** Red de Monitoreo de Calidad del Aire de Bogotá (RMCAB) – Secretaría Distrital
  de Ambiente; cartography from IDECA – Mapa de Referencia de Bogotá, CC BY 4.0.
- **Other cities:** data via [OpenAQ](https://openaq.org), under the license each provider
  specifies; cartography © OpenStreetMap contributors (ODbL).

The repository does not store copies of the data.

## License

The code is released under **MIT or Apache-2.0**, at your option
([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)), as is customary in the
Rust ecosystem. Blog post texts and screenshots are under CC BY 4.0.

Unless you explicitly state otherwise, any contribution you submit for inclusion in Bruma
is dual licensed as above, without any additional terms or conditions.

## Contributing

Everything in this repository is written in **English**: code, comments, documentation,
UI text, log and error messages, and commit messages.
