pub mod perframe_voxel_comp {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/perframe.comp",
    }
}
