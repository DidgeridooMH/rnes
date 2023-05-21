use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, WindowButtons},
};

const NATIVE_RESOLUTION: PhysicalSize<u32> = PhysicalSize::new(256, 240);
const SCALING_FACTOR: u32 = 2;

pub struct MainWindow {
    pub window: Window,
}

impl MainWindow {
    pub async fn new(event_loop: &EventLoop<()>) -> Result<Self, String> {
        let window = match WindowBuilder::new()
            .with_title("RNES")
            .with_inner_size(PhysicalSize::new(
                NATIVE_RESOLUTION.width * SCALING_FACTOR,
                NATIVE_RESOLUTION.height * SCALING_FACTOR,
            ))
            .with_resizable(false)
            .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
            .build(event_loop)
        {
            Ok(w) => w,
            Err(e) => {
                return Err(format!("Unable to open window: {e}"));
            }
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        // TODO: Find best one i guess.
        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .find(|adapter| adapter.is_surface_supported(&surface))
            .unwrap();

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
            .unwrap();

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
        Ok(Self { window })
    }

    // TODO: Make message channel.
    pub fn input(&self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        if let WindowEvent::CloseRequested = event {
            *control_flow = ControlFlow::Exit
        }
    }
}
