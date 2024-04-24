use std::sync::Arc;
use vulkano as vlk;
use vulkano::instance::InstanceCreateFlags;
use vulkano_util::context::VulkanoConfig;

pub struct VxContext {
    pub instance: Arc<vlk::instance::Instance>,
    pub debug_create_info: Option<vlk::instance::debug::DebugUtilsMessengerCreateInfo>,

    // Pass a filter function for physical device selection ->  see default
    pub device_filter_fn: Arc<dyn Fn(&vlk::device::physical::PhysicalDevice) -> bool>,

    //  Pass a priority order function for physical device selection ->  see default
    pub device_priority_fn: Arc<dyn Fn(&vlk::device::physical::PhysicalDevice) -> bool>,

    pub device_extensions: vlk::device::DeviceExtensions,

    pub device_features: vlk::device::Features,

    pub print_device_name: bool,
}

impl Default for VulkanoConfig {
    #[inline]
    fn default() -> Self {
        let device_extensions = vlk::device::DeviceExtensions {
            khr_swapchain: true,
            ..vlk::device::DeviceExtensions::empty()
        };
        VulkanoConfig {
            instance_create_info: vlk::instance::InstanceCreateInfo {
                #[cfg(target_os = "macos")]
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                application_version: vlk::Version::V1_3,
                enabled_extensions: vlk::instance::InstanceExtensions {
                    #[cfg(target_os = "macos")]
                    khr_portability_enumeration: true,
                    ..vlk::instance::InstanceExtensions::empty()
                }
            }
        }
    }
}