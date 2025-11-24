use std::rc::Rc;

///! Draws the same triangle as the `triangle` example, but
///! using the byte based colors.
use miniquad::*;

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    color: [u8; 4],
}

struct Stage {
    pipeline: Pipeline,
    vertices: Buffer,
    indicies: Buffer,
    ctx: Rc<GlContext>,
}

impl Stage {
    pub fn new() -> Stage {
        let ctx = window::new_rendering_backend();

        #[rustfmt::skip]
        let vertices: [Vertex; 3] = [
            Vertex { pos : [ -0.5, -0.5 ], color: [0xFF, 0, 0, 0xFF] },
            Vertex { pos : [  0.5, -0.5 ], color: [0, 0xFF, 0, 0xFF] },
            Vertex { pos : [  0.0,  0.5 ], color: [0, 0, 0xFF, 0xFF] },
        ];
        let vertices = Buffer::new(
            ctx.clone(),
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indicies: [u16; 3] = [0, 1, 2];
        let indicies = Buffer::new(
            ctx.clone(),
            BufferType::IndexBuffer(IndexBufferElementSize::Two),
            BufferUsage::Immutable,
            BufferSource::slice(&indicies),
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
                    vertex_buffers: &[self.vertices.binding(0, 12), self.vertices.binding(8, 12)],
                    index_buffer: &self.indicies,
                    textures: &[],
                    uniform_data: &[],
                }
                .execute()
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

    pub const VERTEX: &str = r#"#version 150
    in vec2 in_pos;
    in lowp uvec4 in_color;

    out lowp vec4 color;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        color = vec4(in_color) / 255.0;
    }"#;

    pub const FRAGMENT: &str = r#"#version 150
    in lowp vec4 color;
    out vec4 frag_color;

    void main() {
        frag_color = color;
    }"#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
        uint4 in_color  [[attribute(1)]];
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
        out.color = float4(v.in_color) / 255.0;

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
                VertexAttribute {
                    gl_pass_as_float: false,
                    ..VertexAttribute::new("in_color", VertexFormat::Byte4)
                },
            ],
        }
    }
}
