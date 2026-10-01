# Bruma

La calidad del aire de Bogotá, dibujada con la GPU.

Bruma es un visualizador web que procesa los datos de las estaciones de la ciudad con
**Rust compilado a WebAssembly** y los dibuja con **WebGPU** (con respaldo en WebGL2).
Es un proyecto abierto de [BogDev](https://bogdev.com.co) y se construye por etapas, cada
una contada en una serie de posts del blog.

> **Estado: Fase 0.** El repositorio, la compilación a Wasm y el primer triángulo en la
> GPU. Todavía no hay datos de calidad del aire.

## Cómo se ve la Fase 0

Un triángulo animado sobre un canvas a pantalla completa. Arriba a la derecha, Bruma dice
qué API gráfica eligió el navegador: **WebGPU** si está disponible, **WebGL2** si no. El
mismo código Rust corre en ambos casos. Si el sistema pide reducir el movimiento
(`prefers-reduced-motion`), la animación queda quieta.

## Estructura

```
bruma/
├── crates/
│   ├── bruma-data/     # modelo y limpieza de datos (Fase 1: descarga e interpolación IDW)
│   └── bruma-render/   # wgpu + shaders WGSL
├── web/                # app web: index.html, estilos y punto de entrada en Rust (Trunk)
├── docs/               # decisiones técnicas
├── LICENSE-MIT
└── LICENSE-APACHE
```

## Requisitos

- Rust estable (1.87 o más reciente) con el target de WebAssembly:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [Trunk](https://trunkrs.dev) 0.21:
  ```bash
  cargo install --locked trunk
  ```

## Desarrollo

```bash
cd web
trunk serve --open        # http://127.0.0.1:8080, con recarga automática
```

Build de producción (queda en `web/dist/`, listo para servir como sitio estático):

```bash
cd web
trunk build --release
```

Pruebas, formato y lint (corren en la máquina, sin navegador):

```bash
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

La prueba de `bruma-render` valida el shader WGSL con naga, así que un error de sintaxis
en el shader aparece en `cargo test` y no al abrir la página.

## Compatibilidad

| Navegador | API que usa Bruma |
| --- | --- |
| Chrome / Edge recientes | WebGPU |
| Safari 26+ (macOS, iOS, iPadOS) | WebGPU |
| Firefox reciente (Windows, macOS) | WebGPU |
| Firefox en Linux y navegadores sin WebGPU | WebGL2 |

## Hoja de ruta

- **Fase 0 — Fundaciones** (este punto): workspace, Trunk, CI y el primer render en WebGPU y WebGL2.
- **Fase 1 — MVP:** datos horarios de PM2.5 de la RMCAB, interpolación IDW y mapa de calor.
- **Fase 2 — v1:** partículas con compute shaders, viento, línea de tiempo de 24 horas y benchmark CPU vs GPU.

Las decisiones de diseño están en [`docs/decisiones.md`](docs/decisiones.md).

## Datos y atribución

A partir de la Fase 1:

- **Calidad del aire:** Red de Monitoreo de Calidad del Aire de Bogotá (RMCAB) – Secretaría
  Distrital de Ambiente. Datos prevalidados; los valores entre estaciones son estimaciones
  por interpolación. Bruma no reemplaza los canales oficiales de alertas.
- **Cartografía:** IDECA – Mapa de Referencia de Bogotá, CC BY 4.0.

El repositorio no guarda copias de los datos.

## Licencia

El código se publica bajo **MIT o Apache-2.0**, a tu elección
([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)), como es costumbre en el
ecosistema Rust. Los textos y capturas de los posts del blog van bajo CC BY 4.0.

Salvo que digas lo contrario, cualquier contribución que envíes para incluir en Bruma se
publica bajo esas mismas dos licencias, sin términos adicionales.
