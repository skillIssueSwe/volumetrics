use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Instant;

use event::WindowEvent;
use glam::{Vec2, vec2, Vec3, vec3};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator as StdCommandBufAlloc, StandardCommandBufferAllocatorCreateInfo as StdCommandBufAllocCreateInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator as StdDescriptorSetAlloc;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::StandardMemoryAllocator as StdMemoryAlloc;
use vulkano::sync::GpuFuture;
use winit::event;
use winit::event::{ElementState, Event, KeyEvent};
use winit::keyboard::{Key, KeyCode, NamedKey};
use crate::perframe_compute::PerframeCompute;
use crate::perframe_render::PerframeRender;
use crate::vulkano_util::window::WindowDescriptor;

pub(crate) struct Engine {
    //  startup_compute_pass: ComputePass,
    perframe_compute: PerframeCompute,
    perframe_render: PerframeRender,
    state: EngineState,
    time_state: TimeState,
    input_state: InputState,
}

impl Engine {
    pub(crate) fn new(
        queue: Arc<Queue>,
        image_format: Format,
        swapchain_image_views: &[Arc<ImageView>],
    ) -> Engine {
        let (memory_alloc, command_buf_alloc, descriptor_set_alloc) = (
            Arc::new(StdMemoryAlloc::new_default(
                queue.device().clone()
            )),
            Arc::new(StdCommandBufAlloc::new(
                queue.device().clone(),
                StdCommandBufAllocCreateInfo {
                    secondary_buffer_count: 32,
                    ..Default::default()
                },
            )),
            Arc::new(StdDescriptorSetAlloc::new(
                queue.device().clone(),
                Default::default(),
            ))
        );

        Engine {
            perframe_compute: PerframeCompute::new(
                queue.clone(),
                memory_alloc.clone(),
                command_buf_alloc.clone(),
                descriptor_set_alloc.clone(),
            ),
            perframe_render: PerframeRender::new(
                queue,
                memory_alloc,
                command_buf_alloc,
                descriptor_set_alloc,
                image_format,
                swapchain_image_views,
            ),
            state: EngineState::default(),
            input_state: InputState::default(),
            time_state: TimeState::default(),
        }
    }

    // runs our compute pipeline and return a future of when the frame is done
    pub(crate) fn compute_frame(&self, image_target: Arc<ImageView>) -> Box<dyn GpuFuture> {
        self.perframe_compute.compute(
            image_target,
            self.state.clone(),
        )
    }

    pub(crate) fn render<F>(
        &self,
        before_future: F,
        computed_view: Arc<ImageView>,
        target: Arc<ImageView>,
        image_index: u32,
    ) -> Box<dyn GpuFuture> where F: GpuFuture + 'static {
        self.perframe_render.render(
            before_future,
            computed_view,
            target,
            image_index,
        )
    }
    pub(crate) fn recreate_framebufs(
        &mut self,
        swapchain_image_views: &[Arc<ImageView>],
    ) {
        self.perframe_render.recreate_framebufs(swapchain_image_views)
    }

    pub(crate) fn on_input_event(&mut self, window_size: [f32; 2], event: &Event<()>) {
        self.input_state.handle_input_event(window_size, event);
    }
}

fn is_key_event_pressed(state: ElementState) -> bool {
    match state {
        ElementState::Pressed => true,
        ElementState::Released => false
    }
}

pub struct VoxelSceneConfig {
    depth: u32,
    width: u32,
    height: u32,
    center: Vec3,
}

impl VoxelSceneConfig {
    pub fn new(
        depth: u32,
        width: u32,
        height: u32,
    ) -> VoxelSceneConfig {
        VoxelSceneConfig {
            depth,
            width,
            height,
            center: glam::vec3(
                (width / 2) as f32,
                (height / 2) as f32,
                (depth / 2) as f32),
        }
    }
}

// Meant to be consumed
struct InputState {
    window_size: [
        f32;2
    ],
    mouse_move_delta: (f32, f32),
    keys_state: HashMap<Keys, bool>,
}

impl InputState {
    // Pass in a function that consumes dx dy into ComputekkjEngineState
    fn consume(&mut self, transform: impl Fn((f32, f32)) -> Result<(), Box<dyn Error>>)
               -> Result<(), Box<dyn Error>> {
        transform(self.mouse_move_delta)?;
        self.mouse_move_delta = (0., 0.);
        Ok(())
    }

    fn on_keyboard_event(&mut self, event: &KeyEvent) {
        match event.logical_key.as_ref() {
            Key::Named(NamedKey::Escape) => { self.keys_state.insert(Keys::ESC, is_key_event_pressed(event.state)); },
            Key::Character("w") => { self.keys_state.insert(Keys::W, is_key_event_pressed(event.state)); }
            Key::Character("a") => { self.keys_state.insert(Keys::A, is_key_event_pressed(event.state)); }
            Key::Character("s") => { self.keys_state.insert(Keys::S, is_key_event_pressed(event.state)); }
            Key::Character("d") => { self.keys_state.insert(Keys::d, is_key_event_pressed(event.state)); }
            _ => ()
        }
    }

    fn on_mouse_event() {}

    fn handle_input_event(&mut self, window_size:[f32; 2], event: &Event<()>) {
        self.window_size = window_size;
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::KeyboardInput { event, .. } => self.on_keyboard_event(event),
                event::DeviceEvent::MouseMotion { delta, .. } => self.on_mouse_input(Keys::MOUSE(delta)),
                _ => {}
            }
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_move_delta: (0., 0.),
            keys_state: HashMap::from([(Keys::ESC, false), (Keys::W, false), (Keys::A, false), (Keys::S, false), (Keys::D, false)])
        }
    }
}

// Hold stuff that isnt input or time
#[derive(Clone)]
pub(crate) struct EngineState {
    cam_position: Vec3,
    cam_angles: Vec3,
    window_size: [f32; 2],
    velocity: f32,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            cam_position: vec3(0., 0., 0.),
            cam_angles: vec3(0., 0., 0.),
            window_size: [WindowDescriptor::default().width, WindowDescriptor::default().height],
            velocity: f32::default(),
        }
    }
}

struct TimeState {
    time: Instant,
    dt: f32,
    dt_sum: f32,
    frame_count: f32,
    average_fps: f32,
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            time: Instant::now(),
            dt: 0.0f32,
            dt_sum: 0.0f32,
            frame_count: 0.0f32,
            average_fps: 0.0f32,
        }
    }
}

impl Default for VoxelSceneConfig {
    fn default() -> Self {
        Self::new(
            32,
            32,
            32,
        )
    }
}

enum Keys {
    MOUSE((f64,f64)),
    W,
    A,
    S,
    D,
    ESC
}