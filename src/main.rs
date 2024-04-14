use std::ffi::CString;
use std::mem::swap;

use ash::{Device, Entry, vk};
use ash::khr::surface::Instance;
use ash::prelude::VkResult;
use ash::vk::{PhysicalDevice, SurfaceKHR, SwapchainKHR};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};

struct VxSurface {
    win_surface_loader: ash::khr::win32_surface::Instance,
    surface_funcs: ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
}

impl VxSurface {
    fn init(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> Result<VxSurface, vk::Result> {
        let surface_create_info =
            if let (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(win32_window_handle)) = (window.raw_display_handle().unwrap(), window.raw_window_handle().unwrap()) {
                ash::vk::Win32SurfaceCreateInfoKHR::default()
                    .hwnd(win32_window_handle.hwnd.get())
                    .hinstance(win32_window_handle.hinstance.ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?.get())
            } else {
                panic!("unsupported display or window handle");
            };

        let win_surface_loader = ash::khr::win32_surface::Instance::new(&entry, &instance);
        let surface_funcs = ash::khr::surface::Instance::new(&entry, &instance);

        let surface = unsafe { win_surface_loader.create_win32_surface(&surface_create_info, None)? };

        Ok(VxSurface { win_surface_loader, surface_funcs, surface })
    }

    fn get_capabilities(&self, physical_device: vk::PhysicalDevice) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        unsafe {
            self.surface_funcs.get_physical_device_surface_capabilities(physical_device, self.surface)
        }
    }

    fn get_present_modes(&self, physical_device: vk::PhysicalDevice) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
        unsafe {
            self.surface_funcs.get_physical_device_surface_present_modes(physical_device, self.surface)
        }
    }

    fn get_formats(&self, physical_device: vk::PhysicalDevice) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        unsafe {
            self.surface_funcs.get_physical_device_surface_formats(physical_device, self.surface)
        }
    }

    fn get_physical_device_surface_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queue_family_index: usize,
    ) -> Result<bool, vk::Result> {
        unsafe {
            self.surface_funcs.get_physical_device_surface_support(
                physical_device,
                queue_family_index as u32,
                self.surface,
            )
        }
    }
}

impl Drop for VxSurface {
    fn drop(&mut self) {
        unsafe {
            self.surface_funcs.destroy_surface(self.surface, None)
        }
    }
}

struct QueueFamilies {
    graphics_q_index: Option<u32>,
    transfer_q_index: Option<u32>,
    compute_q_index: Option<u32>,
}

impl QueueFamilies {
    fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        app_surface: &VxSurface,
    ) -> Result<QueueFamilies, vk::Result> {
        let queue_family_props = unsafe {
            instance.get_physical_device_queue_family_properties(physical_device)
        };

        let mut gqidx = None;
        let mut tqidx = None;
        let mut cqidx = None;

        for (i, it) in queue_family_props.iter().enumerate() {
            if it.queue_count > 0 && it.queue_flags.contains(vk::QueueFlags::GRAPHICS) && gqidx.is_none() &&
                app_surface.get_physical_device_surface_support(physical_device, i)?
            {
                gqidx = Some(i as u32);
            }
            if it.queue_count > 0 && tqidx.is_none() &&
                it.queue_flags.contains(vk::QueueFlags::TRANSFER) &&
                !it.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                tqidx = Some(i as u32);
            }
            if it.queue_count > 0 && cqidx.is_none() &&
                it.queue_flags.contains(vk::QueueFlags::COMPUTE) && !it.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                cqidx = Some(i as u32)
            }
        }
        Ok(
            QueueFamilies {
                graphics_q_index: gqidx,
                transfer_q_index: tqidx,
                compute_q_index: cqidx,
            }
        )
    }
}


struct VxDebugMessenger {
    loader: ash::ext::debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl VxDebugMessenger {
    fn init(entry: &ash::Entry, instance: &ash::Instance) -> Result<VxDebugMessenger, vk::Result> {
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let loader = ash::ext::debug_utils::Instance::new(&entry, &instance);
        let messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None)? };

        Ok(VxDebugMessenger { loader, messenger })
    }
}

impl Drop for VxDebugMessenger {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}

fn init_instance(
    entry: &ash::Entry,
    layer_names: &[&str],
) -> VkResult<ash::Instance> {
    let enginename = CString::new("voxidizer").unwrap();
    let appname = CString::new("voxidizer").unwrap();

    // App Info
    let app_info = vk::ApplicationInfo::default()
        .application_name(&appname)
        .application_version(vk::make_api_version(0, 0, 0, 1))
        .engine_name(&enginename)
        .engine_version(vk::make_api_version(0, 0, 0, 1))
        .api_version(vk::make_api_version(0, 0, 0, 1));

    // Debug layer + instance
    let layer_c_strings: Vec<std::ffi::CString> = layer_names
        .iter()
        .map(|&it| std::ffi::CString::new(it).unwrap())
        .collect();
    let layer_name_ptrs: Vec<*const i8> = layer_c_strings
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let extension_name_ptrs: Vec<*const i8> = vec![
        ash::ext::debug_utils::NAME.as_ptr(),
        ash::khr::surface::NAME.as_ptr(),
        ash::khr::win32_surface::NAME.as_ptr(),
    ];
    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback));

    // Instance
    let instance_create_info = vk::InstanceCreateInfo::default()
        .push_next(&mut debug_create_info)
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_ptrs)
        .enabled_extension_names(&extension_name_ptrs);
    unsafe { entry.create_instance(&instance_create_info, None) }
}

struct VxPhysicalDevice {
    device: vk::PhysicalDevice,
    device_props: vk::PhysicalDeviceProperties,
}

impl VxPhysicalDevice {
    fn init(
        instance: &ash::Instance
    ) -> Result<VxPhysicalDevice, vk::Result> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        for it in devices {
            let props = unsafe { instance.get_physical_device_properties(it) };
            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                return Ok(VxPhysicalDevice { device: it, device_props: props });
            }
        }
        panic!("[DEBUG][PANIC] init failed; no discrete gpu")
    }
}

struct Queues {
    graphics_q: vk::Queue,
    transfer_q: vk::Queue,
    compute_q: vk::Queue,
}

fn init_logical_device(
    instance: &ash::Instance,
    physical_device: &VxPhysicalDevice,
    queue_families: &QueueFamilies,
) -> VkResult<Device> {
    let device_extension_name_ptrs: Vec<*const i8> = vec![ash::khr::swapchain::NAME.as_ptr()];

    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_families.graphics_q_index.unwrap())
            .queue_priorities(&priorities),
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_families.transfer_q_index.unwrap())
            .queue_priorities(&priorities),
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_families.compute_q_index.unwrap())
            .queue_priorities(&priorities)
    ];

    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_ptrs);

    unsafe { instance.create_device(physical_device.device, &device_create_info, None) }
}

fn init_queues(
    queue_families: &QueueFamilies,
    logical_device: &ash::Device,
) -> Result<Queues, vk::Result> {
    let graphics = unsafe { logical_device.get_device_queue(queue_families.graphics_q_index.unwrap(), 0) };
    let transfer = unsafe { logical_device.get_device_queue(queue_families.transfer_q_index.unwrap(), 0) };
    let compute = unsafe { logical_device.get_device_queue(queue_families.compute_q_index.unwrap(), 0) };
    Ok(Queues {
        graphics_q: graphics,
        transfer_q: transfer,
        compute_q: compute,
    })
}


struct VxSwapchain {
    swapchain_fn: ash::khr::swapchain::Device,
    swapchain: SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
}

impl VxSwapchain {
    fn init(
        instance: &ash::Instance,
        physical_device: &VxPhysicalDevice,
        logical_device: &ash::Device,
        app_surface: &VxSurface,
        graphics_q_index: u32,
    ) -> Result<VxSwapchain, vk::Result> {
        let queue = [graphics_q_index];
        let (surface_capabilities, surface_formats) = (
            app_surface.get_capabilities(physical_device.device)?,
            app_surface.get_formats(physical_device.device)?
        );

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(app_surface.surface)
            .min_image_count(
                3.max(surface_capabilities.min_image_count).min(surface_capabilities.max_image_count)
            )
            .image_format(surface_formats.first().unwrap().format)
            .image_color_space(surface_formats.first().unwrap().color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);

        let swapchain_fn = ash::khr::swapchain::Device::new(&instance, &logical_device);
        let swapchain = unsafe { swapchain_fn.create_swapchain(&swapchain_create_info, None)? };

        let images = unsafe { swapchain_fn.get_swapchain_images(swapchain)? };

        let image_views: Vec<vk::ImageView> = images.iter()
            .map(|&it| {
                let subresource_range = vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1);
                let image_view_create_info = vk::ImageViewCreateInfo::default()
                    .image(it)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(vk::Format::B8G8R8A8_UNORM)
                    .subresource_range(subresource_range);
                match unsafe {
                    logical_device
                        .create_image_view(&image_view_create_info, None)
                } {
                    Ok(image_view) => image_view,
                    Err(e) => panic!("Failed to create image view: {:?}", e),
                }
            }).collect::<Vec<vk::ImageView>>();

        Ok(VxSwapchain {
            swapchain_fn,
            swapchain,
            images: images,
            image_views,
        })
    }

    unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
        for it in &self.image_views {
            logical_device.destroy_image_view(*it, None);
        }
        self.swapchain_fn.destroy_swapchain(self.swapchain, None);
    }
}


struct Renderer {
    window: winit::window::Window,
    entry: Entry,
    instance: ash::Instance,
    debug: std::mem::ManuallyDrop<VxDebugMessenger>,
    surfaces: std::mem::ManuallyDrop<VxSurface>,
    physical_device: vk::PhysicalDevice,
    physical_device_props: vk::PhysicalDeviceProperties,
    queue_families: QueueFamilies,
    queues: Queues,
    device: ash::Device,
    swapchain: VxSwapchain,
}

impl Renderer {
    fn init(window: winit::window::Window) -> Result<Renderer, Box<dyn std::error::Error>> {
        let layer_names = vec!["VK_LAYER_KHRONOS_validation"];

        let entry = unsafe { Entry::load()? };
        let instance = init_instance(&entry, &layer_names)?;
        let vk_debug = VxDebugMessenger::init(&entry, &instance)?;


        let physical_device = VxPhysicalDevice::init(&instance)?;

        let app_surface = VxSurface::init(&entry, &instance, &window)?;

        let queue_families = QueueFamilies::init(&instance, physical_device.device, &app_surface)?;


        let device = init_logical_device(&instance, &physical_device, &queue_families)?;

        let queues = init_queues(&queue_families, &device)?;

        let swapchain = VxSwapchain::init(
            &instance,
            &physical_device,
            &device,
            &app_surface,
            queue_families.graphics_q_index.unwrap(),
        )?;

        Ok(Renderer {
            window,
            entry,
            instance,
            debug: std::mem::ManuallyDrop::new(vk_debug),
            surfaces: std::mem::ManuallyDrop::new(app_surface),
            physical_device: physical_device.device,
            physical_device_props: physical_device.device_props,
            queue_families,
            queues,
            device,
            swapchain,
        })
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.cleanup(&self.device);
            self.device.destroy_device(None);
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None);
        };
    }
}

#[allow(deprecated)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize { width: 1440, height: 2560 })
        .build(&event_loop)?;
    let renderer = Renderer::init(window)?;

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            },
            Event::AboutToWait => {
                renderer.window.request_redraw();
            },
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                //render here
            },
            _ => ()
        }
    });
    Ok(())
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}