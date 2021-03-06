use spirv_builder::CompileResult;

use winit::{
    event::{ElementState, VirtualKeyCode, WindowEvent},
    window::Window,
};

enum CurrentRenderPipeline {
    Colour,
    // See also: https://www.youtube.com/watch?v=wh4aWZRtTwU
    Brown,
}

pub struct Pipelines {
    brown_render_pipeline: wgpu::RenderPipeline,
    colour_render_pipeline: wgpu::RenderPipeline,
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    clear_colour: wgpu::Color,

    pipelines: Pipelines,
    current_render_pipeline: CurrentRenderPipeline,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window, shaders: CompileResult) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let pipelines = State::pipelines_from(shaders, &device, &sc_desc);
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            clear_colour: wgpu::Color::default(),
            pipelines,
            current_render_pipeline: CurrentRenderPipeline::Brown,
        }
    }

    fn pipelines_from(
        compilation: CompileResult,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Pipelines {
        let data = std::fs::read(compilation.module.unwrap_single()).unwrap();
        let source = wgpu::util::make_spirv(&data);
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Life Shaders"),
            source,
            flags: wgpu::ShaderFlags::default(),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let brown_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "main_vs",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "main_fs",
                    targets: &[wgpu::ColorTargetState {
                        format: sc_desc.format,
                        write_mask: wgpu::ColorWrite::ALL,
                        blend: Some(wgpu::BlendState::REPLACE),
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });
        let colour_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "main_vs_colour",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "main_fs_colour",
                    targets: &[wgpu::ColorTargetState {
                        format: sc_desc.format,
                        write_mask: wgpu::ColorWrite::ALL,
                        blend: Some(wgpu::BlendState::REPLACE),
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });
        Pipelines {
            brown_render_pipeline,
            colour_render_pipeline,
        }
    }

    pub fn update_pipelines(&mut self, compilation: CompileResult) {
        let pipelines = State::pipelines_from(compilation, &self.device, &self.sc_desc);
        self.pipelines = pipelines;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_colour = wgpu::Color {
                    r: position.x / f64::from(self.size.width),
                    g: position.y / f64::from(self.size.height),
                    b: 0.,
                    a: 0.,
                };
                false
            }
            WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.current_render_pipeline = match self.current_render_pipeline {
                    CurrentRenderPipeline::Colour => CurrentRenderPipeline::Brown,
                    CurrentRenderPipeline::Brown => CurrentRenderPipeline::Colour,
                };
                false
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {}

    fn current_render_pipeline(&self) -> &wgpu::RenderPipeline {
        match self.current_render_pipeline {
            CurrentRenderPipeline::Colour => &self.pipelines.colour_render_pipeline,
            CurrentRenderPipeline::Brown => &self.pipelines.brown_render_pipeline,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_colour),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.current_render_pipeline());
            render_pass.draw(0..3, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit([encoder.finish()]);

        Ok(())
    }
    pub fn recreate_swap_chain(&mut self) {
        self.resize(self.size);
    }
}
