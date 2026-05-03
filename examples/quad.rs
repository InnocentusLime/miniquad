//! A simple rendering example. This example loads a texture from memory
//! and draws a few quads with it. The example should look as follows:
//! https://youtu.be/kksaeWrAT7E

use mimiq::graphics::*;
use mimiq::*;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, vec2};
use image::RgbaImage;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    total_time: Duration,
    ctx: Rc<GlContext>,

    pipeline: Pipeline<ImgVertex, Uniforms, ImageImages<'static>>,
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

    fn init(ctx: Rc<GlContext>, _fs: Rc<dyn FsServer>, _init: ()) -> App {
        #[rustfmt::skip]
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &[
            ImgVertex { v_pos : Vec2 { x: -0.5, y: -0.5 }, v_uv: Vec2 { x: 0., y: 0. } },
            ImgVertex { v_pos : Vec2 { x:  0.5, y: -0.5 }, v_uv: Vec2 { x: 1., y: 0. } },
            ImgVertex { v_pos : Vec2 { x:  0.5, y:  0.5 }, v_uv: Vec2 { x: 1., y: 1. } },
            ImgVertex { v_pos : Vec2 { x: -0.5, y:  0.5 }, v_uv: Vec2 { x: 0., y: 1. } },
        ]);

        #[rustfmt::skip]
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &[
            0, 1, 2,
            0, 2, 3,
        ]);

        let pixels = RgbaImage::from_raw(4, 4, gen_pixels()).unwrap();
        let texture = ctx.new_texture(
            pixels,
            TextureWrap::Clamp,
            FilterMode::Linear,
            FilterMode::Linear,
        );

        let pipeline = ctx.new_pipeline(
            include_str!("shaders/with_offset.vert"),
            include_str!("shaders/basic_texture.frag"),
            PipelineParams {
                blending: Blending::All(BlendFunc {
                    equation: BlendEquation::Add,
                    source: BlendFactor::Value(BlendValue::SrcAlpha),
                    dest: BlendFactor::OneMinusValue(BlendValue::SrcAlpha),
                }),
                ..default_pipeline_params()
            },
        );

        App { pipeline, vertices, indicies, texture, ctx, total_time: Duration::ZERO }
    }
}

impl App {
    fn draw(&mut self) {
        let t = self.total_time.as_secs_f32();

        self.ctx
            .default_pass(Clear::depth_color(Color::BLACK), |_, _| {
                for i in 0..10 {
                    let t = t + i as f32 * 0.3;
                    self.pipeline.draw(
                        0,
                        6,
                        &self.vertices,
                        &self.indicies,
                        &ImageImages { tex: &self.texture },
                        &Uniforms { offset: vec2(t.sin() * 0.5, (t * 3.).cos() * 0.5) },
                    );
                }
            });
    }
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy, Vertex)]
pub struct ImgVertex {
    pub v_pos: Vec2,
    pub v_uv: Vec2,
}

#[derive(Debug, Clone, Copy, ImagesUniformBlock)]
pub struct ImageImages<'a> {
    pub tex: &'a Texture2D,
}

#[repr(C)]
#[derive(Debug, Zeroable, Pod, Clone, Copy, UniformBlock)]
pub struct Uniforms {
    pub offset: Vec2,
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
