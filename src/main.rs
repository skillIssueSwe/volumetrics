use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vulkano::descriptor_set::DescriptorBindingResources::ImageView;

use vulkano::device::Queue;
use vulkano::image::ImageUsage;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::PresentMode;
use vulkano::sync::GpuFuture;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};

use engine::Engine;
use vulkano_util::context::{VulkanoConfig, VulkanoContext};
use vulkano_util::renderer::{DEFAULT_IMAGE_FORMAT, VulkanoWindowRenderer};
use vulkano_util::window::{VulkanoWindows, WindowDescriptor};

mod vulkano_util;
mod engine;
mod shaders;
mod perframe_compute;
mod perframe_render;

fn main() {
    let render_module = RenderModule::default();
    render_module.launch();
}

pub struct RenderModule<'a> {
    event_loop: EventLoop<()>,
    windows: VulkanoWindows,
    render_target_id: usize,
    primary_window_renderer: &'a VulkanoWindowRenderer,
    queue: &'a Arc<Queue>,
    engine: Engine,
    current_image: Some(Arc<ImageView>),
}

impl RenderModule {
    pub fn new(
        window_title: &str,
    ) -> Self {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let context = VulkanoContext::new(VulkanoConfig::default());
        let mut windows = VulkanoWindows::default();
        let render_target_id = 0;
        let _id = windows.create_window(
            &event_loop,
            &context,
            &WindowDescriptor {
                title: window_title.to_string(),
                present_mode: PresentMode::Fifo,
                width: 4000.,
                height: 4000.,
                ..Default::default()
            },
            |_| {},
        );

        let primary_window_renderer = windows.get_primary_renderer_mut().unwrap();

        primary_window_renderer.add_additional_image_view(
            render_target_id,
            DEFAULT_IMAGE_FORMAT,
            ImageUsage::SAMPLED
                | ImageUsage::TRANSFER_DST
                | ImageUsage::STORAGE,
        );

        let graphics_q = context.graphics_queue();

        let mut engine = Engine::new(
            graphics_q.clone(),
            primary_window_renderer.swapchain_format(),
            primary_window_renderer.swapchain_image_views(),
        );
        Self {
            event_loop,
            windows,
            render_target_id,
            primary_window_renderer,
            queue,
            engine,
            current_image: None
        }
    }

    fn launch(mut self)  {

        // TODO Run prepass shaders here

        self.event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            let renderer = self.windows.get_primary_renderer_mut().unwrap(); // TODO see if we can remove this

            if self.on_event(renderer, &event) {
                elwt.exit();
                return;
            }

            self.engine.on_event(renderer.window_size(), &event);
        })
    }

    pub fn on_event(
        &mut self,
        renderer: &mut VulkanoWindowRenderer,
        event: &Event<()>,
    ) -> bool {
        match &event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                return true;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. },
                ..
            } => renderer.resize(),
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => 'redraw: {
                // TODO Tasks to redraw:
                // 1. consume new state to application
                // 2. Compute
                // 3. Render
                // 4. Update things that need to be ready for next run
                // ie. ensure state is correctly consumed - whatever

                // Handle minimization
                match renderer.window_size() {
                    [width, height] => {
                        if width == 0.0 || height == 0.0 {
                            break 'redraw;
                        }
                    }
                }
                // TODO Update engine.rs state
                self.engine.consume_input_state();
                // TODO engine.rs compute
                self.render_frame(renderer);
                // TODO update time
            }
            Event::AboutToWait => renderer.window().request_redraw(),
            _ => (),
        }
        false
        // !compute_engine.instatiated();
    }

    fn render_frame(
        &mut self,
        renderer: &mut VulkanoWindowRenderer,
    ) {
        let before_pipeline_future =
            match renderer.acquire(Some(Duration::from_millis(1)), |swapchain_image_views| {
                self.engine.recreate_framebufs(swapchain_image_views)
            }) {
                Err(e) => {
                    println!("{e}");
                    return;
                }
                Ok(future) => future,
            };

        self.current_image = renderer.get_additional_image_view(self.render_target_id.clone());
        let computepass_future = self.engine.compute_frame(self.current_image.clone())
                                .join(before_pipeline_future);

        let renderpass_future = self.engine.render(
            computepass_future,
            self.current_image.clone(),
            renderer.swapchain_image_view(),
            renderer.image_index()
        );

        renderer.present(renderpass_future, true);
    }
}

impl Default for RenderModule {
    fn default() -> Self {
        RenderModule::new("default")
    }
}


