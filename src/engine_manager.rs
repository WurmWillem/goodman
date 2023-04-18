use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

use crate::engine::Engine;
use crate::prelude::Texture;
use crate::{instances::InstanceRaw, object_data::Vertex};

pub type Vec2 = cgmath::Vector2<f64>;
pub type Vec3 = cgmath::Vector3<f64>;

pub trait Manager {
    fn new(state: &mut Engine, textures: Vec<Texture>) -> Self;
    fn update(&mut self, frame_time: f64, input: &Input);
    fn render(&self, state: &mut Engine);
}

impl Engine {
    pub fn lol(&mut self) {
        //self.background_color;
    }
}

pub struct Input {
    left_mouse_button_pressed: bool,
    d_pressed: bool,
    a_pressed: bool,
    w_pressed: bool,
    s_pressed: bool,
    right_arrow_pressed: bool,
    left_arrow_pressed: bool,
    up_arrow_pressed: bool,
    down_arrow_pressed: bool,
}
impl Input {
    pub fn new() -> Self {
        Self {
            left_mouse_button_pressed: false,
            d_pressed: false,
            a_pressed: false,
            w_pressed: false,
            s_pressed: false,
            right_arrow_pressed: false,
            left_arrow_pressed: false,
            up_arrow_pressed: false,
            down_arrow_pressed: false,
        }
    }
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W => {
                        self.w_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.a_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.s_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.d_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Right => {
                        self.right_arrow_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Left => {
                        self.left_arrow_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Up => {
                        self.up_arrow_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Down => {
                        self.down_arrow_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let is_pressed = *state == ElementState::Pressed;
                match button {
                    MouseButton::Left => {
                        self.left_mouse_button_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
    pub fn reset_buttons(&mut self) {
        if self.left_mouse_button_pressed {
            self.left_mouse_button_pressed = false;
        }
    }
    pub fn is_left_mouse_button_pressed(&self) -> bool {
        self.left_mouse_button_pressed
    }
    pub fn is_d_pressed(&self) -> bool {
        self.d_pressed
    }
    pub fn is_a_pressed(&self) -> bool {
        self.a_pressed
    }
    pub fn is_w_pressed(&self) -> bool {
        self.w_pressed
    }
    pub fn is_s_pressed(&self) -> bool {
        self.s_pressed
    }
    pub fn is_right_arrow_pressed(&self) -> bool {
        self.right_arrow_pressed
    }
    pub fn is_left_arrow_pressed(&self) -> bool {
        self.left_arrow_pressed
    }
    pub fn is_up_arrow_pressed(&self) -> bool {
        self.up_arrow_pressed
    }
    pub fn is_down_arrow_pressed(&self) -> bool {
        self.down_arrow_pressed
    }

}

pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[allow(missing_docs)]
impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color { r, g, b, a}
    }
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
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
