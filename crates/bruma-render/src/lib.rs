//! Bruma rendering with wgpu.
//!
//! The same wgpu API talks to WebGPU when the browser supports it and to
//! WebGL2 when it does not. This crate knows nothing about the DOM: it receives
//! an already created `Surface` (for example, from a `<canvas>`) and handles the rest.

use std::{fmt, sync::Arc};

/// Which graphics API the browser ended up using.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsApi {
    /// The browser's native WebGPU implementation.
    WebGpu,
    /// The WebGL2 fallback.
    WebGl2,
    /// Any other backend (for example, when running outside the browser).
    Other(wgpu::Backend),
}

impl GraphicsApi {
    fn from_backend(backend: wgpu::Backend) -> Self {
        match backend {
            wgpu::Backend::BrowserWebGpu => GraphicsApi::WebGpu,
            wgpu::Backend::Gl => GraphicsApi::WebGl2,
            other => GraphicsApi::Other(other),
        }
    }
}

impl fmt::Display for GraphicsApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphicsApi::WebGpu => f.write_str("WebGPU"),
            GraphicsApi::WebGl2 => f.write_str("WebGL2"),
            GraphicsApi::Other(backend) => write!(f, "{backend:?}"),
        }
    }
}

/// Errors while setting up the GPU.
#[derive(Debug)]
pub enum RenderError {
    /// No GPU adapter compatible with the surface was found.
    NoAdapter(wgpu::RequestAdapterError),
    /// The adapter refused to open a device with the requested limits.
    NoDevice(wgpu::RequestDeviceError),
    /// The surface reports no usable format or alpha mode.
    UnsupportedSurface,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::NoAdapter(e) => write!(f, "no compatible GPU found: {e}"),
            RenderError::NoDevice(e) => write!(f, "could not open the GPU: {e}"),
            RenderError::UnsupportedSurface => {
                f.write_str("the canvas is not compatible with the GPU")
            }
        }
    }
}

impl std::error::Error for RenderError {}

/// Data the shader reads every frame. Must match `Globals` in WGSL.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    time: f32,
    aspect: f32,
    _pad: [f32; 2],
}

// WGSL rounds uniform struct sizes up to a multiple of 16 bytes.
const _: () = assert!(std::mem::size_of::<Globals>().is_multiple_of(16));

/// Dark canvas background: the "sky" everything is drawn on.
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.035,
    g: 0.045,
    b: 0.07,
    a: 1.0,
};

/// Owns the GPU device and draws Bruma into one surface.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    api: GraphicsApi,
}

impl Renderer {
    /// Picks a GPU compatible with `surface`, opens the device and builds the pipeline.
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(RenderError::NoAdapter)?;

        let api = GraphicsApi::from_backend(adapter.get_info().backend);
        log::info!("GPU: {} ({api})", adapter.get_info().name);

        // Request only what WebGL2 guarantees, so the same code runs on both backends.
        let limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("bruma"),
                required_limits: limits,
                ..Default::default()
            })
            .await
            .map_err(RenderError::NoDevice)?;

        let caps = surface.get_capabilities(&adapter);
        // Shader colors are already sRGB-encoded, so they are written to a non-sRGB
        // target unchanged. Choosing it explicitly keeps colors identical on WebGPU
        // and WebGL2, whatever order each backend lists its formats in.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| !format.is_srgb())
            .or_else(|| {
                caps.formats
                    .first()
                    .map(|format| format.remove_srgb_suffix())
            })
            .ok_or(RenderError::UnsupportedSurface)?;
        let alpha_mode = *caps
            .alpha_modes
            .first()
            .ok_or(RenderError::UnsupportedSurface)?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle.wgsl").into()),
        });

        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            globals,
            bind_group,
            api,
        })
    }

    /// Calls `handler` with a readable message whenever the GPU reports an error
    /// nobody caught, or the device is lost. Without this, both fail silently.
    pub fn on_gpu_error(&self, handler: impl Fn(String) + Send + Sync + 'static) {
        let handler = Arc::new(handler);

        let on_error = Arc::clone(&handler);
        self.device
            .on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
                on_error(format!("GPU error: {error}"));
            }));

        self.device
            .set_device_lost_callback(move |reason, message| {
                // `Destroyed` is the normal end of life (e.g. the page is closing).
                if reason != wgpu::DeviceLostReason::Destroyed {
                    handler(format!("the GPU device was lost: {message}"));
                }
            });
    }

    /// Graphics API in use (to show it in the UI).
    pub fn api(&self) -> GraphicsApi {
        self.api
    }

    /// Largest texture side the GPU supports; the canvas must not exceed it.
    pub fn max_dimension(&self) -> u32 {
        self.device.limits().max_texture_dimension_2d
    }

    /// Resizes the surface to the new canvas size, in physical pixels.
    pub fn resize(&mut self, width: u32, height: u32) {
        let max = self.max_dimension();
        let (width, height) = (width.clamp(1, max), height.clamp(1, max));
        if (width, height) == (self.config.width, self.config.height) {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Draws one frame. `time` is the time in seconds since start.
    pub fn render(&mut self, time: f32) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                // Use this frame and reconfigure for the next one.
                self.surface.configure(&self.device, &self.config);
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            other => {
                log::warn!("skipping a frame: {other:?}");
                return;
            }
        };

        let globals = Globals {
            time,
            aspect: self.config.width as f32 / self.config.height as f32,
            _pad: [0.0; 2],
        };
        self.queue
            .write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("triangle"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
    }
}

#[cfg(test)]
mod tests {
    use naga::back::glsl;

    const TRIANGLE: &str = include_str!("shaders/triangle.wgsl");

    fn parse_and_validate(source: &str) -> (naga::Module, naga::valid::ModuleInfo) {
        let module = naga::front::wgsl::parse_str(source).expect("could not parse the WGSL");
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .expect("the WGSL failed validation");
        (module, info)
    }

    /// The shader must be valid WGSL, so a mistake shows up in `cargo test`
    /// instead of only when the page is opened in the browser.
    #[test]
    fn triangle_shader_is_valid_wgsl() {
        parse_and_validate(TRIANGLE);
    }

    /// Every entry point must translate to GLSL ES 3.00; otherwise it cannot
    /// run on the WebGL2 fallback.
    #[test]
    fn triangle_shader_translates_to_webgl2() {
        let (module, info) = parse_and_validate(TRIANGLE);
        let options = glsl::Options {
            version: glsl::Version::Embedded {
                version: 300,
                is_webgl: true,
            },
            ..Default::default()
        };
        for (stage, entry_point) in [
            (naga::ShaderStage::Vertex, "vs_main"),
            (naga::ShaderStage::Fragment, "fs_main"),
        ] {
            let pipeline_options = glsl::PipelineOptions {
                shader_stage: stage,
                entry_point: entry_point.into(),
                multiview: None,
            };
            let mut output = String::new();
            glsl::Writer::new(
                &mut output,
                &module,
                &info,
                &options,
                &pipeline_options,
                naga::proc::BoundsCheckPolicies::default(),
            )
            .and_then(|mut writer| writer.write())
            .unwrap_or_else(|e| panic!("{entry_point} does not translate to GLSL ES 3.00: {e}"));
        }
    }

    /// `Globals` in Rust must have exactly the size naga computes for its WGSL twin.
    #[test]
    fn globals_layout_matches_wgsl() {
        let (module, _) = parse_and_validate(TRIANGLE);
        let mut layouter = naga::proc::Layouter::default();
        layouter
            .update(module.to_ctx())
            .expect("could not compute the WGSL layout");
        let (handle, _) = module
            .types
            .iter()
            .find(|(_, ty)| ty.name.as_deref() == Some("Globals"))
            .expect("struct Globals not found in the shader");
        assert_eq!(
            layouter[handle].size as usize,
            std::mem::size_of::<super::Globals>()
        );
    }
}
