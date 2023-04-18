use std::collections::HashMap;
use std::time::Instant;

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::camera::{Camera, self};
use crate::engine::Engine;
use crate::instances::Instance;
use crate::prelude::{Input, Vec2, Color};
use crate::texture::{Texture, self};
use crate::{instances::InstanceRaw, object_data::Vertex};

impl Engine {
    pub fn create_texture(&mut self, bytes: &[u8], label: &str) -> Texture {
        let tex = Texture::from_bytes(&self.device, &self.queue, bytes, label)
            .unwrap_or_else(|_| panic!("Could not create {label} texture"));

        let texture_bind_group_layout = super::texture::create_bind_group_layout(&self.device);
        let texture_bind_group =
            texture::create_bind_group(&self.device, &texture_bind_group_layout, &tex);

        self.texture_bind_groups
            .insert(tex.label.clone(), texture_bind_group);
        tex
    }

    pub fn get_frame_time(&self) -> f64 {
        self.last_frame.elapsed().as_secs_f64()
    }
    pub fn get_average_tps(&mut self) -> u32 {
        (self.frames_passed_this_sec as f64 / self.frame_time_this_sec) as u32
    }
    pub fn get_target_fps(&self) -> Option<u32> {
        self.target_fps
    }
    pub fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
    pub fn get_time_since_last_render(&self) -> f64 {
        self.time_since_last_render
    }

    pub fn set_fps(&mut self, fps: Option<u32>) {
        self.target_fps = fps;
    }
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = wgpu::Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }

    pub async fn new(size: Vec2, event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new() //350 - 1;
            .with_inner_size(PhysicalSize::new(size.x, size.y))
            .build(event_loop)
            .expect("Failed to build window");

        let size = window.inner_size();

        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(), // sudo sysctl dev.i915.perf_stream_paranoid=0
        });

        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.expect("Failed to init surface");

        let adapter = super::engine_manager::create_adapter(&instance, &surface).await;
        let (device, queue) = super::engine_manager::create_device_and_queue(&adapter).await;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = super::engine_manager::create_surface_format(&surface_caps);

        let config = super::engine_manager::create_config(&surface_format, size, &surface_caps);
        surface.configure(&device, &config);

        let texture_bind_group_layout = super::texture::create_bind_group_layout(&device);
        let texture_bind_groups = HashMap::new();

        let camera = Camera::new(false);
        let camera_buffer = camera::create_buffer(&device, camera.uniform);
        let camera_bind_group_layout = camera::create_bind_group_layout(&device);
        let camera_bind_group =
            camera::create_bind_group(&device, &camera_buffer, &camera_bind_group_layout);

        let instances = vec![];
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = super::instances::create_buffer(&device, &instance_data);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = super::engine_manager::create_render_pipeline_layout(
            &device,
            &texture_bind_group_layout,
            &camera_bind_group_layout,
        );
        let render_pipeline = super::engine_manager::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &shader,
            &config,
        );

        let (vertex_buffer, index_buffer) = super::object_data::create_buffers(&device);

        let background_color = wgpu::Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        };

        Self {
            window,
            background_color,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            camera,
            camera_bind_group,
            camera_buffer,
            instance_buffer,
            instances,
            instances_raw: instance_data,
            input: Input::new(),
            last_frame: Instant::now(),
            frame_time_this_sec: 0.,
            frames_passed_this_sec: 0,
            time_since_last_render: 0.,
            target_fps: None,
            //target_tps: 5700,
            instances_drawn: 0,
            bind_group_indexes: HashMap::new(),
            texture_bind_groups,
        }
    }
}

pub async fn create_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        })
        .await
        .expect("Failed to create adapter")
}

pub async fn create_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .expect("failed to create device or queue")
}

pub fn create_surface_format(surface_caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.describe().srgb)
        .unwrap_or(surface_caps.formats[0])
}

pub fn create_config(
    surface_format: &wgpu::TextureFormat,
    size: PhysicalSize<u32>,
    surface_caps: &wgpu::SurfaceCapabilities,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: *surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    }
}

pub fn create_render_pipeline_layout(
    device: &wgpu::Device,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[texture_bind_group_layout, camera_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    render_pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc(), InstanceRaw::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
