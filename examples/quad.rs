use std::rc::Rc;

use glam::{vec2, Vec2};
///! A simple rendering example. This example loads a texture from memory
///! and draws a few quads with it. The example should look as follows:
///! https://youtu.be/kksaeWrAT7E
use miniquad::*;

#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

struct Stage {
    ctx: Rc<GlContext>,

    pipeline: Pipeline,
    vertices: Buffer,
    indicies: Buffer,
    texture: Texture,
}

impl Stage {
    pub fn new() -> Stage {
        let ctx = window::new_rendering_backend();

        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertices = Buffer::new(
            ctx.clone(),
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indicies: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let indicies = Buffer::new(
            ctx.clone(),
            BufferType::IndexBuffer(IndexBufferElementSize::Two),
            BufferUsage::Immutable,
            BufferSource::slice(&indicies),
        );

        let pixels: [u8; 4 * 4 * 4] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
            0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let texture = Texture::new(
            ctx.clone(),
            TextureSource::Bytes(&pixels),
            TextureParams {
                format: TextureFormat::RGBA8,
                width: 4,
                height: 4,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                allocate_mipmaps: false,
            },
        );

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
            vertices,
            indicies,
            texture,
            ctx,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        let t = date::now();

        self.ctx
            .perform_default_render_pass(PassAction::default(), || {
                for i in 0..10 {
                    let t = t + i as f64 * 0.3;
                    let uniforms = shader::Uniforms {
                        offset: vec2(t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5),
                    };
                    DrawCall {
                        pipeline: &self.pipeline,
                        base_element: 0,
                        num_elements: 6,
                        vertex_buffers: &[
                            self.vertices.binding(0, 16),
                            self.vertices.binding(8, 16),
                        ],
                        index_buffer: &self.indicies,
                        textures: &[&self.texture],
                        uniform_data: bytemuck::bytes_of(&uniforms),
                    }
                    .execute();
                }
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
    use bytemuck::{Pod, Zeroable};
    use glam::Vec2;
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos + offset, 0, 1);
        texcoord = in_uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float2 offset;
    };

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
        float2 in_uv    [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float2 uv       [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(
      Vertex v [[stage_in]], 
      constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy + uniforms.offset, 0.0, 1.0);
        out.uv = v.in_uv;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return tex.sample(texSmplr, in.uv);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
            attributes: vec![
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
        }
    }

    #[repr(C)]
    #[derive(Zeroable, Pod, Clone, Copy)]
    pub struct Uniforms {
        pub offset: Vec2,
    }
}
