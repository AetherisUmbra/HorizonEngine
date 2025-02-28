use crate::render_context::RenderContext;
use crate::uniform::UniformBufferObject;
use crate::vertex::Vertex;
use anyhow::Result;
use hmath::matrix::Matrix4x4;
use hmath::vector::Vector3;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, RenderingAttachmentInfo, RenderingInfo,
};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, RasterizationState};
use vulkano::pipeline::graphics::subpass::PipelineRenderingCreateInfo;
use vulkano::pipeline::graphics::vertex_input::{Vertex as VulkanoVertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo
};
use vulkano::render_pass::{AttachmentLoadOp, AttachmentStoreOp};
use vulkano::shader::spirv::bytes_to_words;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::swapchain::{
    acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::GpuFuture;
use vulkano::{sync, Validated, Version, VulkanError, VulkanLibrary};
use winit::event_loop::EventLoop;
use winit::window::Window;

pub struct Renderer {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    vertex_buffer: Subbuffer<[Vertex]>,
    index_buffer: Subbuffer<[u16]>,
    render_context: Option<RenderContext>,
    uniform_buffer_allocator: Option<SubbufferAllocator>,
    current_view_matrix: Matrix4x4,
    current_projection_matrix: Matrix4x4,
}

fn load_shader(device: Arc<Device>, name: &str) -> Arc<ShaderModule> {
    let path = format!("res/shaders/{}.spv", name);
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open SPIR-V file {}: {}", path, err);
            panic!("Failed to open SPIR-V file");
        }
    };

    let mut buffer = Vec::new();
    if let Err(err) = file.read_to_end(&mut buffer) {
        eprintln!("Failed to read SPIR-V file {}: {}", path, err);
        panic!("Failed to read SPIR-V file");
    }

    let words = match bytes_to_words(&buffer) {
        Ok(words) => words,
        Err(err) => {
            eprintln!("Failed to convert bytes to words for {}: {}", path, err);
            panic!("Failed to convert bytes to words");
        }
    };

    let create_info = ShaderModuleCreateInfo::new(&words);
    unsafe { ShaderModule::new(device, create_info).expect("Failed to create shader module") }
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let library = VulkanLibrary::new()?;

        let required_extensions = Surface::required_extensions(event_loop)?;

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                enabled_layers: vec!["VK_LAYER_KHRONOS_validation".into()],
                ..Default::default()
            },
        )?;

        let mut device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()?
            .filter(|p| {
                p.api_version() >= Version::V1_3 || p.supported_extensions().khr_dynamic_rendering
            })
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, event_loop).unwrap()
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable physical device found");

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        if physical_device.api_version() < Version::V1_3 {
            device_extensions.khr_dynamic_rendering = true;
        }

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],

                enabled_extensions: device_extensions,

                enabled_features: DeviceFeatures {
                    dynamic_rendering: true,
                    ..DeviceFeatures::empty()
                },

                ..Default::default()
            },
        )?;

        let queue = queues.next().unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let vertices = [
            // Front face
            Vertex { position: Vector3::new(-0.5, -0.5,  0.5), color: Vector3::new(1.0, 0.0, 0.0) }, // 0
            Vertex { position: Vector3::new( 0.5, -0.5,  0.5), color: Vector3::new(1.0, 0.0, 0.0) }, // 1
            Vertex { position: Vector3::new( 0.5,  0.5,  0.5), color: Vector3::new(1.0, 0.0, 0.0) }, // 2
            Vertex { position: Vector3::new(-0.5,  0.5,  0.5), color: Vector3::new(1.0, 0.0, 0.0) }, // 3
            
            // Back face
            Vertex { position: Vector3::new(-0.5, -0.5, -0.5), color: Vector3::new(0.0, 1.0, 0.0) }, // 4
            Vertex { position: Vector3::new( 0.5, -0.5, -0.5), color: Vector3::new(0.0, 1.0, 0.0) }, // 5
            Vertex { position: Vector3::new( 0.5,  0.5, -0.5), color: Vector3::new(0.0, 1.0, 0.0) }, // 6
            Vertex { position: Vector3::new(-0.5,  0.5, -0.5), color: Vector3::new(0.0, 1.0, 0.0) }, // 7
        ];

        let indices: [u16; 36] = [
            // Front face
            0, 2, 1,  0, 3, 2,
            // Back face
            5, 7, 4,  5, 6, 7,
            // Top face
            3, 6, 2,  3, 7, 6,
            // Bottom face
            4, 1, 5,  4, 0, 1,
            // Right face
            1, 6, 5,  1, 2, 6,
            // Left face
            4, 3, 0,  4, 7, 3
        ];

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )?;

        let index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )?;

        Ok(Renderer {
            instance,
            device,
            queue,
            command_buffer_allocator,
            descriptor_set_allocator,
            memory_allocator,
            vertex_buffer,
            index_buffer,
            render_context: None,
            uniform_buffer_allocator: None,
            current_view_matrix: Matrix4x4::identity(),
            current_projection_matrix: Matrix4x4::identity(),
        })
    }

    pub fn create_render_context(&mut self, window: Arc<Window>) -> Result<()> {
        let surface = Surface::from_window(self.instance.clone(), Arc::clone(&window))?;
        let window_size = window.inner_size();

        let (swapchain, images) = {
            let surface_capabilities = self
                .device
                .physical_device()
                .surface_capabilities(&surface, Default::default())?;

            let formats = self
                .device
                .physical_device()
                .surface_formats(&surface, Default::default())?;

            let image_format = formats
                .iter()
                .find(|(format, _)| {
                    matches!(
                        format,
                        vulkano::format::Format::R16G16B16A16_SFLOAT
                            | vulkano::format::Format::A2B10G10R10_UNORM_PACK32
                    )
                })
                .map(|(format, _)| *format)
                .unwrap_or(formats[0].0);

            Swapchain::new(
                self.device.clone(),
                surface,
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window_size.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )?
        };

        let attachment_image_views = window_size_dependent_setup(&images);

        let vs = load_shader(self.device.clone(), "shader.vert");
        let fs = load_shader(self.device.clone(), "shader.frag");

        let depth_buffer = ImageView::new_default(
            Image::new(
                self.memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D16_UNORM,
                    extent: images[0].extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )?;

        let pipeline = {
            let vs_entry_point = vs.entry_point("main").unwrap();
            let fs_entry_point = fs.entry_point("main").unwrap();

            let vertex_input_state = match Vertex::per_vertex().definition(&vs_entry_point) {
                Ok(state) => state,
                Err(e) => {
                    eprintln!("Pipeline vertex input state creation failed: {:?}", e);
                    panic!("Pipeline vertex input state failed!");
                }
            };

            let stages = [
                PipelineShaderStageCreateInfo::new(vs_entry_point),
                PipelineShaderStageCreateInfo::new(fs_entry_point),
            ];

            let descriptor_set_layouts = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.device.clone())?;
            
            let layout = PipelineLayout::new(
                self.device.clone(),
                descriptor_set_layouts,
            )?;

            let subpass = PipelineRenderingCreateInfo {
                color_attachment_formats: vec![Some(swapchain.image_format())],
                depth_attachment_format: Some(Format::D16_UNORM),
                ..Default::default()
            };

            let rasterization_state = RasterizationState {
                cull_mode: CullMode::Back,
                ..Default::default()
            };

            GraphicsPipeline::new(
                self.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(rasterization_state),
                    multisample_state: Some(MultisampleState::default()),
                    depth_stencil_state: Some(DepthStencilState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.color_attachment_formats.len() as u32,
                        ColorBlendAttachmentState::default(),
                    )),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(subpass.into()),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )?
        };

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_size.into(),
            depth_range: 0.0..=1.0,
        };

        let recreate_swapchain = false;

        let previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.render_context = Some(RenderContext {
            swapchain,
            attachment_image_views,
            pipeline,
            viewport,
            recreate_swapchain,
            previous_frame_end,
            depth_buffer,
        });

        let uniform_buffer_allocator = SubbufferAllocator::new(
            self.memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );
        
        self.uniform_buffer_allocator = Some(uniform_buffer_allocator);

        Ok(())
    }

    pub fn resize(&mut self) {
        if let Some(render_context) = self.render_context.as_mut() {
            render_context.recreate_swapchain = true;
        }
    }

    pub fn draw(&mut self, window: Arc<Window>) {
        let window_size = window.inner_size();

        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        let render_context = if let Some(ref mut context) = self.render_context {
            context
        } else {
            return;
        };

        render_context
            .previous_frame_end
            .as_mut()
            .unwrap()
            .cleanup_finished();

        if render_context.recreate_swapchain {
            let (new_swapchain, new_images) = render_context
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: window_size.into(),
                    ..render_context.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            render_context.swapchain = new_swapchain;
            render_context.attachment_image_views = window_size_dependent_setup(&new_images);
            render_context.viewport.extent = window_size.into();
            render_context.recreate_swapchain = false;
        }

        let uniform_buffer = if let Some(allocator) = &self.uniform_buffer_allocator {
            let uniform_data = UniformBufferObject::new(self.current_view_matrix, self.current_projection_matrix);
            
            let buffer = allocator.allocate_sized().unwrap();
            *buffer.write().unwrap() = uniform_data;

            buffer
        } else {
            return;
        };
        
        let layout = &render_context.pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(render_context.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    render_context.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            render_context.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_rendering(RenderingInfo {
                color_attachments: vec![Some(RenderingAttachmentInfo {
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    clear_value: Some([0.0, 0.0, 0.0, 1.0].into()),
                    ..RenderingAttachmentInfo::image_view(
                        render_context.attachment_image_views[image_index as usize].clone(),
                    )
                })],
                depth_attachment: Some(RenderingAttachmentInfo {
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::DontCare,
                    clear_value: Some([1.0, 0.0, 0.0, 0.0].into()),
                    ..RenderingAttachmentInfo::image_view(render_context.depth_buffer.clone())
                }),
                ..Default::default()
            })
            .unwrap()
            .set_viewport(0, [render_context.viewport.clone()].into_iter().collect())
            .unwrap()
            .bind_pipeline_graphics(render_context.pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                render_context.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap()
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .unwrap()
            .bind_index_buffer(self.index_buffer.clone())
            .unwrap();

        unsafe { builder.draw_indexed(self.index_buffer.len() as u32, 1, 0, 0, 0) }.unwrap();

        builder.end_rendering().unwrap();

        let command_buffer = builder.build().unwrap();

        let future = render_context
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    render_context.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                render_context.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                render_context.recreate_swapchain = true;
                render_context.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                render_context.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        }
    }

    pub fn set_view_matrix(&mut self, view: &Matrix4x4) {
        self.current_view_matrix = *view;
    }

    pub fn set_projection_matrix(&mut self, projection: &Matrix4x4) {
        self.current_projection_matrix = *projection;
    }

    pub fn set_camera_matrices(&mut self, view: &Matrix4x4, projection: &Matrix4x4) {
        self.current_view_matrix = *view;
        self.current_projection_matrix = *projection;
    }
}

fn window_size_dependent_setup(images: &[Arc<Image>]) -> Vec<Arc<ImageView>> {
    images
        .iter()
        .map(|image| ImageView::new_default(image.clone()).unwrap())
        .collect::<Vec<_>>()
}
