use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec4, Vec2, Vec4};
///! Just draws a static triangle with different vertex colors assigned
///! to each corner:
///! * left -- red
///! * right -- green
///! * top -- blue
use miniquad::*;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    color: Vec4,
}

struct Stage {
    pipeline: Pipeline,
    vertices: Buffer<Vertex>,
    indicies: IndexBuffer<u16>,
    ctx: Rc<GlContext>,
}

impl Stage {
    pub fn new() -> Stage {
        let ctx = window::new_rendering_backend();

        #[rustfmt::skip]
        let vertices = [
            Vertex { pos: vec2(-0.5, -0.5), color: vec4(1., 0., 0., 1.) },
            Vertex { pos: vec2(0.5, -0.5), color: vec4(0., 1., 0., 1.) },
            Vertex { pos: vec2(0.0,  0.5), color: vec4(0., 0., 1., 1.) },
        ];
        let vertices = Buffer::new(ctx.clone(), BufferUsage::Immutable, &vertices);

        let indicies = [0, 1, 2];
        let indicies = IndexBuffer::new(ctx.clone(), BufferUsage::Immutable, &indicies);

        let pipeline = Pipeline::new(
            ctx.clone(),
            match ctx.info().backend {
                Backend::OpenGl => ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: shader::FRAGMENT,
                },
                Backend::Metal => ShaderSource::Msl {
                    program: shader::METAL,
                },
            },
            shader::meta(),
            PipelineParams::default(),
        )
        .unwrap();

        Stage {
            pipeline,
            indicies,
            vertices,
            ctx,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.ctx
            .perform_default_render_pass(PassAction::default(), || {
                DrawCall {
                    pipeline: &self.pipeline,
                    base_element: 0,
                    num_elements: 3,
                    vertex_buffers: &[self.vertices.binding(0), self.vertices.binding(8)],
                    index_buffer: &self.indicies,
                    textures: &[],
                    uniform_data: &[],
                }
                .execute();
            });
    }
}

fn main() {
    let mut conf = conf::Conf::default();
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    conf.platform.apple_gfx_api = if metal {
        conf::AppleGfxApi::Metal
    } else {
        conf::AppleGfxApi::OpenGl
    };

    miniquad::start(conf, move || Box::new(Stage::new()));
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec4 in_color;

    varying lowp vec4 color;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        color = in_color;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;

    void main() {
        gl_FragColor = color;
    }"#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
        float4 in_color [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy, 0.0, 1.0);
        out.color = v.in_color;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]])
    {
        return in.color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: vec![],
            attributes: vec![
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_color", VertexFormat::Float4),
            ],
        }
    }
}
