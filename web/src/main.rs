//! Punto de entrada web de Bruma (Fase 0).
//!
//! Busca el `<canvas>`, detecta si el navegador tiene WebGPU (si no, usa WebGL2),
//! y dibuja un triángulo en cada cuadro con `requestAnimationFrame`.

use std::{cell::RefCell, rc::Rc};

use bruma_render::Renderer;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{HtmlCanvasElement, Window};

/// La función de cada cuadro; se guarda aquí para poder volver a pedirla desde sí misma.
type FrameCallback = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);

    wasm_bindgen_futures::spawn_local(async {
        if let Err(message) = start().await {
            log::error!("{message}");
            set_text("bruma-status", &message);
        }
    });
}

async fn start() -> Result<(), String> {
    let window = web_sys::window().ok_or("no hay ventana del navegador")?;
    let document = window.document().ok_or("no hay documento")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("bruma-canvas")
        .ok_or("no se encontró el canvas #bruma-canvas")?
        .dyn_into()
        .map_err(|_| "#bruma-canvas no es un <canvas>")?;

    // Pedimos WebGPU y WebGL2; wgpu descarta WebGPU si el navegador no lo tiene,
    // antes de tocar el canvas (un canvas solo puede tener un tipo de contexto).
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL;
    let instance = wgpu::util::new_instance_with_webgpu_detection(descriptor).await;

    let surface = create_surface(&instance, &canvas)?;
    let (width, height) = physical_size(&window, &canvas);
    let renderer = Renderer::new(&instance, surface, width, height)
        .await
        .map_err(|e| {
            format!("Bruma necesita WebGPU o WebGL2 y este navegador no los ofrece ({e}).")
        })?;

    log::info!("dibujando con {}", renderer.api());
    set_text("bruma-api", &renderer.api().to_string());
    set_text("bruma-status", "");

    let reduced_motion = window
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|query| query.matches());

    run_loop(window, canvas, renderer, reduced_motion);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn create_surface(
    instance: &wgpu::Instance,
    canvas: &HtmlCanvasElement,
) -> Result<wgpu::Surface<'static>, String> {
    instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
        .map_err(|e| format!("no se pudo usar el canvas: {e}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn create_surface(
    _instance: &wgpu::Instance,
    _canvas: &HtmlCanvasElement,
) -> Result<wgpu::Surface<'static>, String> {
    Err("Bruma web solo corre en el navegador (compila con Trunk)".into())
}

/// Tamaño del canvas en píxeles físicos (tamaño CSS × densidad de pantalla).
fn physical_size(window: &Window, canvas: &HtmlCanvasElement) -> (u32, u32) {
    let ratio = window.device_pixel_ratio();
    let width = (f64::from(canvas.client_width()) * ratio).round() as u32;
    let height = (f64::from(canvas.client_height()) * ratio).round() as u32;
    (width.max(1), height.max(1))
}

/// Dibuja en cada cuadro y sigue el tamaño del canvas si cambia la ventana.
fn run_loop(
    window: Window,
    canvas: HtmlCanvasElement,
    mut renderer: Renderer,
    reduced_motion: bool,
) {
    let performance = window.performance();
    let started_at = performance.as_ref().map_or(0.0, |p| p.now());

    let frame: FrameCallback = Rc::new(RefCell::new(None));
    let next = frame.clone();
    let loop_window = window.clone();

    *frame.borrow_mut() = Some(Closure::new(move || {
        let max = renderer.max_dimension();
        let (width, height) = physical_size(&loop_window, &canvas);
        let (width, height) = (width.min(max), height.min(max));
        if canvas.width() != width || canvas.height() != height {
            canvas.set_width(width);
            canvas.set_height(height);
        }
        renderer.resize(width, height);

        // Con "reducir movimiento" activado, la imagen queda quieta.
        let time = match (&performance, reduced_motion) {
            (Some(p), false) => ((p.now() - started_at) / 1000.0) as f32,
            _ => 0.0,
        };
        renderer.render(time);

        if let Some(callback) = next.borrow().as_ref() {
            request_frame(&loop_window, callback);
        }
    }));

    if let Some(callback) = frame.borrow().as_ref() {
        request_frame(&window, callback);
    }
}

fn request_frame(window: &Window, callback: &Closure<dyn FnMut()>) {
    if let Err(e) = window.request_animation_frame(callback.as_ref().unchecked_ref()) {
        log::error!("requestAnimationFrame falló: {e:?}");
    }
}

fn set_text(id: &str, text: &str) {
    if let Some(element) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    {
        element.set_text_content(Some(text));
    }
}
