use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator as StdCommandBufAlloc;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator as StdDescriptorSetAlloc;
use vulkano::device::Queue;
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::StandardMemoryAllocator as StdMemoryAlloc;
use vulkano::pipeline::{ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::{PipelineDescriptorSetLayoutCreateInfo};
use vulkano::sync::GpuFuture;
use crate::engine::VoxelSceneConfig;
use crate::engine::EngineState;
use crate::shaders::compute_shaders::perframe_voxel_comp;

pub struct PerframeCompute {
    queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
    memory_alloc: Arc<StdMemoryAlloc>,
    command_buf_alloc: Arc<StdCommandBufAlloc>,
    descriptor_set_alloc: Arc<StdDescriptorSetAlloc>,
    scene_config: VoxelSceneConfig,
}


impl PerframeCompute {
    pub fn new(
        queue: Arc<Queue>,
        memory_alloc: Arc<StdMemoryAlloc>,
        command_buf_alloc: Arc<StdCommandBufAlloc>,
        descriptor_set_alloc: Arc<StdDescriptorSetAlloc>,
    ) -> PerframeCompute {
        let pipeline = {
            let device = queue.device();
            let compute_shader = perframe_voxel_comp::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let stage = PipelineShaderStageCreateInfo::new(compute_shader);
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap()
            ).unwrap();

            ComputePipeline::new(
                device.clone(),
                None,
                ComputePipelineCreateInfo::stage_layout(stage, layout)
            ).unwrap()
        };

        PerframeCompute {
            queue,
            pipeline,
            memory_alloc,
            command_buf_alloc,
            descriptor_set_alloc,
            scene_config: VoxelSceneConfig::default()
        }
    }

    pub fn compute(
        &self,
        image_view: Arc<ImageView>,
        engine_state: EngineState,
    ) -> Box<dyn GpuFuture> {

    }
}


