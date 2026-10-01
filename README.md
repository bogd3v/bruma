# Bruma

La calidad del aire de tu ciudad, dibujada con la GPU.

Bruma es un visualizador web que toma los datos de las estaciones de monitoreo de una
ciudad, los procesa con **Rust compilado a WebAssembly** y los dibuja con **WebGPU** (con
respaldo en WebGL2). Funciona con cualquier ciudad que tenga estaciones con datos
públicos: cada ciudad es una configuración y cada red de monitoreo, un adaptador.
**Bogotá es la primera ciudad** y el caso de referencia.

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
│   ├── bruma-data/     # modelo y limpieza de datos, común a todas las ciudades (Fase 1: adaptadores e IDW)
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
- **Fase 1 — MVP:** configuración de ciudades, adaptadores de datos (RMCAB para Bogotá y OpenAQ como fuente genérica), interpolación IDW y mapa de calor.
- **Fase 2 — v1:** partículas con compute shaders, viento, línea de tiempo de 24 horas, más ciudades y benchmark CPU vs GPU.

## Agregar tu ciudad

A partir de la Fase 1 bastará con describir la ciudad (nombre, zona del mapa, contorno) y
elegir de dónde salen sus datos:

- **OpenAQ**, si la ciudad tiene estaciones con datos recientes ahí: no hace falta escribir código.
- **Un adaptador propio**, si la red local publica sus datos por otra vía (como la RMCAB en Bogotá).

Si quieres ver tu ciudad en Bruma, abre un issue con el nombre de la ciudad y dónde
publica sus datos de calidad del aire.

Las decisiones de diseño están en [`docs/decisiones.md`](docs/decisiones.md).

## Datos y atribución

A partir de la Fase 1, cada ciudad muestra la atribución de sus propias fuentes. En todos
los casos, los valores entre estaciones son estimaciones por interpolación y Bruma no
reemplaza los canales oficiales de alertas.

- **Bogotá:** Red de Monitoreo de Calidad del Aire de Bogotá (RMCAB) – Secretaría Distrital
  de Ambiente; cartografía de IDECA – Mapa de Referencia de Bogotá, CC BY 4.0.
- **Otras ciudades:** datos vía [OpenAQ](https://openaq.org), con la licencia que indique
  cada proveedor; cartografía de © colaboradores de OpenStreetMap (ODbL).

El repositorio no guarda copias de los datos.

## Licencia

El código se publica bajo **MIT o Apache-2.0**, a tu elección
([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)), como es costumbre en el
ecosistema Rust. Los textos y capturas de los posts del blog van bajo CC BY 4.0.

Salvo que digas lo contrario, cualquier contribución que envíes para incluir en Bruma se
publica bajo esas mismas dos licencias, sin términos adicionales.
