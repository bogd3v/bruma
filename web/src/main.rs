//! Bruma web entry point (Phase 0).
//!
//! Finds the `<canvas>`, detects whether the browser has WebGPU (falling back to
//! WebGL2), and draws a triangle every frame with `requestAnimationFrame`.

use std::{cell::RefCell, rc::Rc};

use bruma_render::Renderer;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{HtmlCanvasElement, Window};

/// The per-frame callback; stored here so it can request itself again.
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
    let window = web_sys::window().ok_or("no browser window")?;
    let document = window.document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("bruma-canvas")
        .ok_or("canvas #bruma-canvas not found")?
        .dyn_into()
        .map_err(|_| "#bruma-canvas is not a <canvas>")?;

    // Ask for WebGPU and WebGL2; wgpu drops WebGPU if the browser lacks it,
    // before touching the canvas (a canvas can only hold one context type).
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL;
    let instance = wgpu::util::new_instance_with_webgpu_detection(descriptor).await;

    let surface = create_surface(&instance, &canvas)?;
    let (width, height) = physical_size(&window, &canvas);
    let renderer = Renderer::new(&instance, surface, width, height)
        .await
        .map_err(|e| {
            format!("Bruma needs WebGPU or WebGL2 and this browser offers neither ({e}).")
        })?;

    log::info!("drawing with {}", renderer.api());
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
        .map_err(|e| format!("could not use the canvas: {e}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn create_surface(
    _instance: &wgpu::Instance,
    _canvas: &HtmlCanvasElement,
) -> Result<wgpu::Surface<'static>, String> {
    Err("Bruma web only runs in the browser (build it with Trunk)".into())
}

/// Canvas size in physical pixels (CSS size × device pixel ratio).
fn physical_size(window: &Window, canvas: &HtmlCanvasElement) -> (u32, u32) {
    let ratio = window.device_pixel_ratio();
    let width = (f64::from(canvas.client_width()) * ratio).round() as u32;
    let height = (f64::from(canvas.client_height()) * ratio).round() as u32;
    (width.max(1), height.max(1))
}

/// Draws every frame and follows the canvas size when the window changes.
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

        // With "reduce motion" enabled, the image stays still.
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
        log::error!("requestAnimationFrame failed: {e:?}");
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
