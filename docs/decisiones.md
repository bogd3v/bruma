# Decisiones técnicas

Registro corto de las decisiones de Bruma y su porqué. Cada una puede cambiar; si cambia,
se anota aquí en vez de borrar la anterior.

## Lenguaje y compilación

- **Todo el núcleo en Rust**, compilado a WebAssembly. El JavaScript queda en lo mínimo que
  genera wasm-bindgen.
- **Trunk** como herramienta de compilación: Bruma es una aplicación web, no una librería
  para npm. Trunk parte de `web/index.html`, compila a Wasm, genera el enlace con
  wasm-bindgen y pasa `wasm-opt`. wasm-pack quedaría solo si algún día `bruma-data` se
  publica como paquete npm.
- **Perfil de release** orientado a tamaño (`opt-level = "z"`, LTO, `panic = "abort"`): en
  la web, la descarga inicial pesa más que unos milisegundos de CPU.

## Gráficos

- **wgpu** como única API de render: habla WebGPU cuando el navegador lo soporta y WebGL2
  cuando no.
- **Detección antes de tocar el canvas:** un `<canvas>` solo admite un tipo de contexto, así
  que primero se comprueba si hay WebGPU (`new_instance_with_webgpu_detection`) y luego se
  crea la superficie.
- **Límites de WebGL2 para ambos backends:** se piden los límites que WebGL2 garantiza, para
  que el mismo código corra igual en los dos. Las funciones exclusivas de WebGPU (compute
  shaders) llegan en la Fase 2 con su propia ruta.
- **Shaders validados en `cargo test`** con naga.
- **Accesibilidad:** con `prefers-reduced-motion` la animación se detiene.

## Datos

- **Fuente:** reporte horario público de la RMCAB (Secretaría Distrital de Ambiente),
  consultado una vez por hora desde un proxy propio, nunca desde el navegador de cada
  visitante. OpenAQ se descartó para tiempo real: sus datos de Bogotá dejaron de
  actualizarse en 2022 y no incluyen viento.
- **Respaldo para viento:** Open-Meteo (datos modelados, etiquetados como tales).
- **Cartografía:** IDECA – Mapa de Referencia de Bogotá (CC BY 4.0); OpenStreetMap como
  alternativa.
- **Limpieza:** se descartan los códigos de "sin dato" (`-9999`) y los valores fuera de un
  rango plausible por variable. Los rangos son provisionales hasta tener la documentación
  oficial de la red.

## Infraestructura

- **Dokploy** en el mismo VPS del blog, sin costos adicionales.
- **Un solo subdominio**, `bruma.bogdev.com.co`: el sitio en `/` y el proxy de datos en
  `/api`, enrutados por Traefik (mismo origen, sin CORS).
- El registro DNS se declara en el repositorio de infraestructura de BogDev (Terraform).
