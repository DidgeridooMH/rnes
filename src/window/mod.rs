use wgpu::{
    util::DeviceExt, Buffer, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration, Texture,
};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod dialog_parent;
mod keymap_view;
mod keymap_window;
mod menu;
mod uniform;
mod vertex;

pub use dialog_parent::DialogParent;
pub use keymap_window::{preview as keymap_preview, KeyMapEvent, KeyMapWindow};
pub use menu::{AppMenu, MenuAction};

use uniform::WindowUniform;

pub const NATIVE_RESOLUTION: PhysicalSize<u32> = PhysicalSize::new(256, 240);
pub const BYTES_PER_PIXEL: usize = 4;
const SCALING_FACTOR: u32 = 3;

pub struct MainWindow {
    pub window: Window,
    surface: Surface,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    uniform_buffer: Buffer,
    uniform_bind_group: wgpu::BindGroup,
    screen_texture: Texture,
    texture_bind_group: wgpu::BindGroup,
    config: SurfaceConfiguration,
    rom_name: Option<String>,
}

/// Backends tried first: the native graphics API for each platform.
#[cfg(target_os = "windows")]
const PREFERRED_BACKENDS: wgpu::Backends = wgpu::Backends::DX12;
#[cfg(target_os = "macos")]
const PREFERRED_BACKENDS: wgpu::Backends = wgpu::Backends::METAL;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const PREFERRED_BACKENDS: wgpu::Backends = wgpu::Backends::VULKAN;

/// Creates a surface for `window` and finds an adapter that can present to
/// it using only the given backends.
fn find_adapter(
    window: &Window,
    backends: wgpu::Backends,
) -> Option<(wgpu::Surface, wgpu::Adapter)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        dx12_shader_compiler: Default::default(),
    });
    // SAFETY: the window outlives the surface (both are owned by MainWindow).
    let surface = unsafe { instance.create_surface(window) }.ok()?;
    let adapter = instance
        .enumerate_adapters(backends)
        .find(|adapter| adapter.is_surface_supported(&surface))?;
    Some((surface, adapter))
}

impl MainWindow {
    /// Makes the window visible. Call after the first frame has been drawn.
    pub fn show(&self) {
        self.window.set_visible(true);
    }

    pub async fn new(event_loop: &EventLoop<()>) -> Result<Self, String> {
        let window = match WindowBuilder::new()
            .with_title("RNES")
            .with_inner_size(PhysicalSize::new(
                NATIVE_RESOLUTION.width * SCALING_FACTOR,
                (NATIVE_RESOLUTION.height) * SCALING_FACTOR,
            ))
            .with_resize_increments(NATIVE_RESOLUTION)
            // Shown by `show()` once the first frame is drawn, so the window
            // doesn't sit blank while the GPU is initialized.
            .with_visible(false)
            .build(event_loop)
        {
            Ok(w) => w,
            Err(e) => {
                return Err(format!("Unable to open window: {e}"));
            }
        };

        let (surface, adapter) = match wgpu::util::backend_bits_from_env() {
            // An explicit WGPU_BACKEND is honored as-is.
            Some(backends) => find_adapter(&window, backends),
            // Otherwise try the platform's native API first. Initializing every
            // backend is slow: creating a Vulkan instance loads all registered
            // Vulkan layers (overlays from Steam, OBS, etc.), which can take
            // over a second even though DX12 ends up being used on Windows.
            None => find_adapter(&window, PREFERRED_BACKENDS)
                .or_else(|| find_adapter(&window, wgpu::Backends::all())),
        }
        .ok_or("No compatible graphics adapter found")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(|e| format!("Unable to create graphics device: {e}"))?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Vertices"),
            contents: bytemuck::cast_slice(vertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Aspect Ratio Buffer"),
            contents: bytemuck::cast_slice(&[WindowUniform::new(
                NATIVE_RESOLUTION.width as f32 / NATIVE_RESOLUTION.height as f32,
                window.inner_size().width as f32 / window.inner_size().height as f32,
            )]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let window_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let window_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &window_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let screen_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: NATIVE_RESOLUTION.width,
                height: NATIVE_RESOLUTION.height - 16,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Screen Texture"),
            view_formats: &[],
        });

        let screen_texture_view =
            screen_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let screen_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            });
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&screen_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&screen_sampler),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("screen_shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &window_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            uniform_bind_group: window_bind_group,
            screen_texture,
            texture_bind_group,
            config,
            rom_name: None,
        })
    }

    /// Sets the window's client area to the default scaled NES resolution.
    /// Call after attaching a menu bar so the menu doesn't eat into it.
    pub fn reset_size(&self) {
        self.window.set_inner_size(PhysicalSize::new(
            NATIVE_RESOLUTION.width * SCALING_FACTOR,
            NATIVE_RESOLUTION.height * SCALING_FACTOR,
        ));
    }

    /// A handle for making native dialogs modal to this window.
    pub fn dialog_parent(&self) -> DialogParent<'_> {
        DialogParent::new(&self.window)
    }

    /// Sets the loaded ROM name shown in the title bar (None when no ROM).
    pub fn set_rom_name(&mut self, name: Option<String>) {
        self.rom_name = name;
        match &self.rom_name {
            Some(name) => self.window.set_title(&format!("RNES - {name}")),
            None => self.window.set_title("RNES - File > Open ROM (Ctrl+O)"),
        }
    }

    pub fn set_subtitle(&self, subtitle: &str) {
        match &self.rom_name {
            Some(name) => self.window.set_title(&format!("RNES - {name} ({subtitle} fps)")),
            None => self.window.set_title(&format!("RNES: {subtitle}")),
        }
    }

    pub fn input(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(**new_inner_size);
            }
            _ => {}
        }
    }

    pub fn render(&mut self, screen_buffer: &[u32]) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..(vertex::VERTICES.len() as u32), 0..1);
        }

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.screen_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&screen_buffer[(8 * NATIVE_RESOLUTION.width as usize)..]),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * NATIVE_RESOLUTION.width),
                rows_per_image: Some(NATIVE_RESOLUTION.height - 16),
            },
            wgpu::Extent3d {
                width: NATIVE_RESOLUTION.width,
                height: NATIVE_RESOLUTION.height - 16,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.queue.write_buffer(
                &self.uniform_buffer,
                0,
                bytemuck::cast_slice(&[WindowUniform::new(
                    NATIVE_RESOLUTION.width as f32 / NATIVE_RESOLUTION.height as f32,
                    self.window.inner_size().width as f32 / self.window.inner_size().height as f32,
                )]),
            )
        }
    }
}
