use std::task::Context;

use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub trait Game {
    fn init(engine: &mut Engine) -> Self;
    fn resize(&mut self, engine: &mut Engine);
    fn update(&mut self, engine: &mut Engine);
    fn draw(&mut self, engine: &mut Engine);
}
pub struct Settings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
}

pub struct Engine {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    display: Display,
    textures: Vec<wgpu::Texture>,
    samplers: Vec<wgpu::Sampler>,
    buffers: Vec<wgpu::Buffer>,
}

impl Engine {
    pub async fn new(settings: Settings, event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let display_internals = DisplayInternals::new(&settings, event_loop, &instance)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&display_internals.surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await?;
        let display = display_internals.build(&adapter, &device);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            display,
            textures: vec![],
            samplers: vec![],
            buffers: vec![],
        })
    }

    fn resize_display(&mut self, size: PhysicalSize<u32>) {
        self.display.resize(&self.device, size);
    }
}

pub struct DisplayInternals {
    window: Window,
    surface: wgpu::Surface,
}

impl DisplayInternals {
    fn new(
        settings: &Settings,
        event_loop: &EventLoop<()>,
        instance: &wgpu::Instance,
    ) -> anyhow::Result<Self> {
        let mut resolution = settings.resolution;

        if settings.fullscreen {
            // TODO: figure out what monitor the user is on
            // event_loop.available_monitors().
        }

        if resolution.0 == 0 {
            resolution.0 = 1;
        }
        if resolution.1 == 0 {
            resolution.1 = 1;
        }

        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(resolution.0, resolution.1))
            .build(&event_loop)?;
        let surface = unsafe { instance.create_surface(&window) };
        Ok(Self { window, surface })
    }

    fn build(self, adapter: &wgpu::Adapter, device: &wgpu::Device) -> Display {
        let size = self.window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: self.surface.get_preferred_format(adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        self.surface.configure(device, &config);
        Display {
            internals: self,
            config,
        }
    }
}

pub struct Display {
    internals: DisplayInternals,
    config: wgpu::SurfaceConfiguration,
}

impl Display {
    fn resize(&mut self, device: &wgpu::Device, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.internals.surface.configure(device, &self.config);
    }

    fn get_current_frame(&mut self) -> wgpu::SurfaceTexture {
        self.internals
            .surface
            .get_current_frame()
            .expect("Failed to acquire next swap chain texure")
            .output
    }
}

pub async fn run<G: Game + 'static>(settings: Settings) -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let mut engine = Engine::new(settings, &event_loop).await?;
    let mut game = G::init(&mut engine);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
                ..
            } => {
                if engine.display.internals.window.id() == window_id {
                    engine.resize_display(size);
                    game.resize(&mut engine);
                }
            }
            Event::RedrawRequested(window_id) => {
                if engine.display.internals.window.id() == window_id {
                    // TODO: the draw function should get a frame to draw
                    // to and the data for textures and such
                    // game.draw(&mut engine);
                    let frame = engine.display.get_current_frame();
                    // TODO: this may be a point for optimization
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = engine
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    {
                        let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: true,
                                },
                            }],
                            depth_stencil_attachment: None,
                        });
                    }

                    engine.queue.submit(Some(encoder.finish()));
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if engine.display.internals.window.id() == window_id {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    })
}
