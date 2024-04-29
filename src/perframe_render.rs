use std::sync::Arc;

use vulkano::command_buffer::allocator::StandardCommandBufferAllocator as StdCommandBufAlloc;
use vulkano::descriptor_set::{allocator::StandardDescriptorSetAllocator as StdDescriptorSetAlloc, PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::StandardMemoryAllocator as StdMemoryAlloc;
use vulkano::pipeline::{DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sync::GpuFuture;
use crate::shaders::fragment_shaders::quad_painter_frag;
use crate::shaders::vertex_shaders::quad_vert;

pub struct PerframeRender {
    graphics_q: Arc<Queue>,
    draw_renderpass: Arc<RenderPass>,
    paint_quad_pipeline: QuadPainterPipeline,
    command_buf_alloc: Arc<StdCommandBufAlloc>,
    framebufs: Vec<Arc<Framebuffer>>,
}


impl PerframeRender {
    pub fn new(
        graphics_q: Arc<Queue>,
        memory_alloc: Arc<StdMemoryAlloc>,
        command_buf_alloc: Arc<StdCommandBufAlloc>,
        descriptor_set_alloc: Arc<StdDescriptorSetAlloc>,
        target_format: Format,
        image_views: &[Arc<ImageView>],
    ) -> PerframeRender {
        let draw_renderpass = vulkano::single_pass_renderpass!(
            graphics_q.device().clone(),
            attachments: {
                color: {
                    format: target_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        ).unwrap();

        let subpass = Subpass::from(draw_renderpass.clone(), 0).unwrap();
        let paint_quad_pipeline = QuadPainterPipeline::new(
            graphics_q.clone(),
            subpass,
            command_buf_alloc.clone(),
            descriptor_set_alloc
        );

        PerframeRender {
            graphics_q,
            draw_renderpass: draw_renderpass.clone(),
            paint_quad_pipeline,
            command_buf_alloc,
            framebufs: PerframeRender::create_framebufs(image_views, draw_renderpass)
        }
    }

    pub fn render<F>(
        &self,
        before_future: F,
        image_view: Arc<ImageView>,
        target: Arc<ImageView>,
        image_index: u32,
    ) -> Box<dyn GpuFuture> where F: GpuFuture + 'static {


    }

    // TODO
    pub fn recreate_framebufs(&mut self, swapchain_images_views: &[Arc<ImageView>]) {
        self.framebufs = PerframeRender::create_framebufs(swapchain_images_views, self.draw_renderpass.clone())
    }

    fn create_framebufs(
        swapchain_image_views: &[Arc<ImageView>],
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        swapchain_image_views.iter()
                             .map(|iv| {
                                 Framebuffer::new(
                                     render_pass.clone(),
                                     FramebufferCreateInfo {
                                         attachments: vec![iv.clone()],
                                         ..Default::default()
                                     },
                                 ).unwrap()
                             })
                             .collect::<Vec<_>>()
    }
}



pub struct QuadPainterPipeline {
    graphics_q: Arc<Queue>,
    subpass: Subpass,
    pipeline: Arc<GraphicsPipeline>,
    command_buf_alloc: Arc<StdCommandBufAlloc>,
    descriptor_set_alloc: Arc<StdDescriptorSetAlloc>,
}

impl QuadPainterPipeline {
    pub fn new(
        graphics_q: Arc<Queue>,
        subpass: Subpass,
        command_buf_alloc: Arc<StdCommandBufAlloc>,
        descriptor_set_alloc: Arc<StdDescriptorSetAlloc>,
    ) -> Self {
        let device = graphics_q.device();
        let quad_vert = quad_vert::load(device.clone())
            .expect("failed to create vert shader mod")
            .entry_point("main")
            .expect("shader entry fn not found");
        let quad_frag = quad_painter_frag::load(device.clone())
            .expect("failed to create frag shader mod")
            .entry_point("main")
            .expect("shader entry fn not found");
        let pipeline = {
            let device = graphics_q.device();
            let stages = [
                PipelineShaderStageCreateInfo::new(quad_vert),
                PipelineShaderStageCreateInfo::new(quad_frag)
            ];
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap(),
            ).unwrap();

            GraphicsPipeline::new(
                device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(VertexInputState::default()),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(RasterizationState::default()),
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState::default(),
                    )),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(subpass.clone().into()),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            ).unwrap()
        };

        QuadPainterPipeline {
            graphics_q,
            subpass,
            pipeline,
            command_buf_alloc,
            descriptor_set_alloc
        }
    }

    fn create_descriptor_set(&self, image: Arc<ImageView>) -> Arc<PersistentDescriptorSet> {
        let layout = &self.pipeline.layout().set_layouts()[0];
        let sampler = Sampler::new(
            self.graphics_q.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                mipmap_mode: SamplerMipmapMode::Linear,
                ..Default::default()
            }
        ).unwrap();

        let pipeline_layout = self.pipeline.layout();
        let desc_set_layouts = pipeline_layout.set_layouts();

        let descriptor_set_layout = desc_set_layouts
            .get(0)
            .unwrap();

        PersistentDescriptorSet::new(
            &*self.descriptor_set_alloc.clone(),
            layout.clone(),
            [
                WriteDescriptorSet::sampler(0, sampler),
                WriteDescriptorSet::image_view(1, image),
            ],
            []
        ).unwrap()
    }
}


