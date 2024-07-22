use std::{borrow::Cow, iter::once, mem::size_of};
use std::sync::Arc;
use tokio::runtime::Runtime;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    state: Option<GameState<'a>>,
}

struct GameState<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
}

impl<'a> GameState<'a> {
    async fn new(window: Arc<Window>) -> GameState<'a> {

        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Failed to initialised surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                ..Default::default()
            })
        .await
        .expect("Failed to get adaptor");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor::default(),
                None,
            )
            .await
            .expect("Failed to load device");

        let swapchain_cap = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_cap.formats[0];
        
        let config = surface.get_default_config(&adapter,size.width, size.height,).unwrap();
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label:None,
            source:wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label:None,
            bind_group_layouts:&[],
            push_constant_ranges:&[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            multiview: None,
            layout: Some(&pipeline_layout), 
            vertex: wgpu::VertexState{
                module: &shader,
                entry_point:"vs_main",
                buffers:&[],
                compilation_options: Default::default()
        },
        fragment:Some(wgpu::FragmentState{
            module:&shader,
            entry_point:"fs_main",
            compilation_options:wgpu::PipelineCompilationOptions::default(),
            targets:&[Some(wgpu::ColorTargetState{
                format:swapchain_format,
                blend:None,
                write_mask:wgpu::ColorWrites::all(),
            })],
        }),

        label:None,
        primitive:wgpu::PrimitiveState{
            topology:wgpu::PrimitiveTopology::TriangleList, 
            ..Default::default()
        },
        depth_stencil:None,
        multisample:wgpu::MultisampleState::default(),
                });
        Self {
            surface,
            device,
            queue,
            size,
            render_pipeline
        }
    }

 fn render(&mut self) {
     let output = self.surface.get_current_texture().ok().unwrap();
     let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
     let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
     {    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            color_attachments:&[Some(wgpu::RenderPassColorAttachment{
                resolve_target:None, 
                view:&view,
                ops: wgpu::Operations{
                load: wgpu::LoadOp::Clear(wgpu::Color { r:0.5,g:0.5,b:0.5,a:1.0} ),    
                store: wgpu::StoreOp::Store,
            },
            })], ..Default::default()
     });
   render_pass.set_pipeline(&self.render_pipeline);
   render_pass.draw(0..3,0..1);
 }
   self.queue.submit(Some(encoder.finish()));
   output.present();
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("WGPU Hello Triangle").with_inner_size(winit::dpi::LogicalSize::new(1280.0,720.0));
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("failed to get window attributes"),
            );
            self.window = Some(window.clone());
            let rt = Runtime::new().expect("Failed to get runtime");
            let state = GameState::new(window.clone());
            let state = rt.block_on(state);
            self.state = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
            self.state.as_mut().unwrap().render();
                self.window
                    .as_ref()
                    .expect("failed to redraw window")
                    .request_redraw();
            }
            _ => (),
        }
    }
}


fn main() {
    let event_loop = EventLoop::new().expect("Failed to get event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
