//! A simple rendering example. This example loads a texture from memory
//! and draws a few quads with it. The example should look as follows:
//! https://youtu.be/kksaeWrAT7E

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, vec2};
use image::RgbaImage;
use mimiq::*;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    total_time: Duration,
    ctx: Rc<GlContext>,

    pipeline: Pipeline<Meta>,
    vertices: VertexBuffer<ImgVertex>,
    indicies: IndexBuffer,
    texture: Texture2D,
}

impl EventHandler<()> for App {
    fn update(&mut self, dt: Duration) {
        self.total_time += dt;
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: FsServerHandle, _init: ()) -> App {
        #[rustfmt::skip]
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &[
            ImgVertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            ImgVertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            ImgVertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            ImgVertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ]);

        #[rustfmt::skip]
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &[
            0, 1, 2,
            0, 2, 3,
        ]);

        let pixels = RgbaImage::from_raw(4, 4, gen_pixels()).unwrap();
        let texture = ctx.new_texture(
            pixels,
            Texture2DParams {
                internal_format: Texture2DFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
            },
        );

        let pipeline = ctx.new_pipeline();

        App { pipeline, vertices, indicies, texture, ctx, total_time: Duration::ZERO }
    }
}

impl App {
    fn draw(&mut self) {
        let t = self.total_time.as_secs_f32();

        self.ctx.default_pass(Clear::depth_color(BLACK), |_, _| {
            for i in 0..10 {
                let t = t + i as f32 * 0.3;
                self.ctx.draw(DrawCall {
                    pipeline: &self.pipeline,
                    base_element: 0,
                    num_elements: 6,
                    vertex_buffer: &self.vertices,
                    index_buffer: &self.indicies,
                    images: &self.texture,
                    uniforms: &Uniforms { offset: vec2(t.sin() * 0.5, (t * 3.).cos() * 0.5) },
                });
            }
        });
    }
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
pub struct ImgVertex {
    pub pos: Vec2,
    pub uv: Vec2,
}

impl Vertex for ImgVertex {
    const LAYOUT: &'static [VertexField] =
        &[attribute_of!(ImgVertex, pos), attribute_of!(ImgVertex, uv)];
}

pub struct Meta;

impl PipelineMeta for Meta {
    const VERTEX_SHADER: &str = include_str!("shaders/with_offset.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/basic_texture.frag");

    const IMAGES_NAMES: &str = "tex";
    type Images = Texture2D;
    type Vertex = ImgVertex;
    type Uniforms = Uniforms;
    const PARAMS: PipelineParams = PipelineParams {
        blending: Blending::All(BlendFunc {
            equation: BlendEquation::Add,
            source: BlendFactor::Value(BlendValue::SrcAlpha),
            dest: BlendFactor::OneMinusValue(BlendValue::SrcAlpha),
        }),
        ..default_pipeline_params()
    };
}

#[repr(C)]
#[derive(Debug, Zeroable, Pod, Clone, Copy)]
pub struct Uniforms {
    pub offset: Vec2,
}

impl UniformBlock for Uniforms {
    const FIELDS: &'static [UniformField] = &[uniform_of!(Uniforms, offset)];
}

#[rustfmt::skip]
fn gen_pixels() -> Vec<u8> {
    vec![
        0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0x00,
        0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
    ]
}
