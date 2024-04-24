mod VxContext;

use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use vulkano as vlk;
use vulkano_util as vlku;
use vulkano_util::context::VulkanoConfig;
use vulkano_util::renderer::DEFAULT_IMAGE_FORMAT;
use vulkano_util::window::VulkanoWindows;
use winit::event_loop::ControlFlow;


fn main() -> Result<(), impl Error>{

}

struct RenderModule<'a>{
    event_loop: winit::event_loop::EventLoop<()>,
    vcontext: vlku::context::VulkanoContext,
    windows: vlku::window::VulkanoWindows,
    render_target_id: usize,
    primary_window_renderer: &'a vlku::renderer::VulkanoWindowRenderer,
    graphics_q: &'a Arc<vlk::device::Queue>,
    compute_q: &'a Arc<vlk::device::Queue>,
    // app
    compute_engine: ComputeEngine
}

impl RenderModule {
    fn new() -> Self {
        let event_loop = winit::event_loop::EventLoopWindowTarget::new().unwrap();
        let context = vlku::context::VulkanoContext::new(VulkanoConfig::default());
        let mut windows = VulkanoWindows::default();
        let render_target_id = 0;
        let _id = windows.create_window(
            &event_loop,
            &context,
            &vlku::window::WindowDescriptor {
                title: "default".to_string(),
                present_mode: vlk::swapchain::PresentMode::Fifo,
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
            vlk::image::ImageUsage::SAMPLED
                | vlk::image::ImageUsage::TRANSFER_DST
                | vlk::image::ImageUsage::STORAGE
        );

        let graphics_q = context.graphics_queue();

        let mut compute_engine = ComputeEngine::new(
            graphics_q.clone(),
            primary_window_renderer.swapchain_format(),
            primary_window_renderer.swapchain_image_views()
        );
        Self {
            event_loop,
            vcontext: context,
            windows,
            render_target_id,
            primary_window_renderer,
            graphics_q,
            compute_q: context.compute_queue(),
            compute_engine
        }
    }

    fn launch(mut self) -> Result<(), impl Error> {
        self.event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            let renderer = self.windows.get_primary_renderer_mut(); // TODO see if we can remove this

            match on_event(renderer, &event, &mut self.compute_engine, self.render_target_id) {

            }
        })
    }

    // Random thought, try using graphics queue to get render_target


    fn on_event(
        renderer: &mut vlku::renderer::VulkanoWindowRenderer,
        event: &winit::event::Event<()>,
        compute_engine: &mut ComputeEngine,
        render_target_id: usize,
    )
}

struct ComputeEngine {
    compute_pipeline: VxComputePipeline,

    render_frame_pass: RenderFramePass,
    state: ComputeEngineState,
    time_state: ComputeEngineTime,
    input_state: MouseInputState,
}

impl ComputeEngine {
    fn new(
        main_q: Arc<vlk::device::Queue>,
        image_format: vlk::format::Format,
        swapchain_image_views: &[Arc<vlk::image::view::ImageView>]
    ) -> Self {

        Self {

        }
    }
}
struct MouseInputState {

}

// Hold stuff that isnt input or time
struct ComputeEngineState {

}

struct ComputeEngineTime {
    time: Instant,
    dt: f32,
    dt_sum: f32,
    frame_cnt: f32,
    avg_fps: f32,
}


struct RenderFramePass {

}

struct VxComputePipeline {

}