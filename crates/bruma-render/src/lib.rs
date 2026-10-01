//! Renderizado de Bruma con wgpu.
//!
//! La misma API de wgpu habla con WebGPU cuando el navegador lo soporta y con
//! WebGL2 cuando no. Este crate no sabe nada del DOM: recibe una `Surface` ya
//! creada (por ejemplo, a partir de un `<canvas>`) y se encarga del resto.

use std::fmt;

/// Qué API gráfica terminó usando el navegador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsApi {
    WebGpu,
    WebGl2,
    /// Otro backend (por ejemplo, al ejecutar fuera del navegador).
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

/// Errores al preparar la GPU.
#[derive(Debug)]
pub enum RenderError {
    NoAdapter(wgpu::RequestAdapterError),
    NoDevice(wgpu::RequestDeviceError),
    UnsupportedSurface,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::NoAdapter(e) => write!(f, "no se encontró una GPU compatible: {e}"),
            RenderError::NoDevice(e) => write!(f, "no se pudo abrir la GPU: {e}"),
            RenderError::UnsupportedSurface => f.write_str("el canvas no es compatible con la GPU"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Datos que el shader lee en cada cuadro. Debe coincidir con `Globals` en WGSL.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    time: f32,
    aspect: f32,
    _pad: [f32; 2],
}

/// Fondo oscuro del canvas: el "cielo" sobre el que se dibuja.
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.035,
    g: 0.045,
    b: 0.07,
    a: 1.0,
};

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
    /// Elige una GPU compatible con `surface`, abre el dispositivo y prepara el pipeline.
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

        // Pedimos solo lo que WebGL2 garantiza; así el mismo código corre en ambos.
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
        let format = *caps
            .formats
            .first()
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

    /// API gráfica en uso (para mostrarla en la interfaz).
    pub fn api(&self) -> GraphicsApi {
        self.api
    }

    /// Lado máximo de textura que admite la GPU; el canvas no debe pasarse de aquí.
    pub fn max_dimension(&self) -> u32 {
        self.device.limits().max_texture_dimension_2d
    }

    /// Ajusta la superficie al nuevo tamaño del canvas, en píxeles físicos.
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

    /// Dibuja un cuadro. `time` es el tiempo en segundos desde el inicio.
    pub fn render(&mut self, time: f32) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                // Se usa este cuadro y se reconfigura para el siguiente.
                self.surface.configure(&self.device, &self.config);
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            other => {
                log::warn!("se omite un cuadro: {other:?}");
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
    /// El shader debe ser WGSL válido; así un error aparece en `cargo test`
    /// y no recién al abrir la página en el navegador.
    #[test]
    fn triangle_shader_is_valid_wgsl() {
        let source = include_str!("shaders/triangle.wgsl");
        let module = naga::front::wgsl::parse_str(source).expect("el WGSL no se pudo leer");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .expect("el WGSL no pasó la validación");
    }
}
