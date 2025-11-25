use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
///! An offscreen render example. Draws a cube that has
///! images of rotating cubes on each side. Should look like this:
///! https://youtu.be/isKW3nQ-jW4
use miniquad::*;

use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct CubeVert {
    pub pos: Vec3,
    pub color: Vec4,
    pub uv: Vec2,
}

struct Stage {
    vertices_cube: Buffer<CubeVert>,
    indicies_cube: IndexBuffer,
    display_pipeline: Pipeline,
    offscreen_pipeline: Pipeline,
    offscreen_pass: RenderPass,
    rx: f32,
    ry: f32,
    ctx: Rc<GlContext>,
}

impl Stage {
    pub fn new() -> Stage {
        let ctx = window::new_rendering_backend();
        let color_img = Texture::new(
            ctx.clone(),
            TextureSource::Empty,
            TextureParams {
                width: 256,
                height: 256,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = Texture::new(
            ctx.clone(),
            TextureSource::Empty,
            TextureParams {
                width: 256,
                height: 256,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        );

        let offscreen_pass = RenderPass::new(vec![color_img], None, Some(depth_img));

        #[rustfmt::skip]
        let vertices_cube = &[
            CubeVert { pos: vec3(-1.0, -1.0, -1.0), color: vec4(1.0, 0.5, 0.5, 1.0), uv: vec2(0.0, 0.0) },
            CubeVert { pos: vec3(1.0, -1.0, -1.0), color: vec4(1.0, 0.5, 0.5, 1.0),  uv: vec2(1.0, 0.0) },
            CubeVert { pos: vec3(1.0,  1.0, -1.0), color: vec4(1.0, 0.5, 0.5, 1.0), uv: vec2(1.0, 1.0) },
            CubeVert { pos: vec3(-1.0,  1.0, -1.0),  color: vec4(1.0, 0.5, 0.5, 1.0), uv: vec2(0.0, 1.0) }, 

            CubeVert { pos: vec3(-1.0, -1.0,  1.0), color: vec4(0.5, 1.0, 0.5, 1.0), uv: vec2(0.0, 0.0) },
            CubeVert { pos: vec3(1.0, -1.0,  1.0),  color: vec4(0.5, 1.0, 0.5, 1.0), uv: vec2(1.0, 0.0) },
            CubeVert { pos: vec3(1.0,  1.0,  1.0),  color: vec4(0.5, 1.0, 0.5, 1.0),  uv: vec2(1.0, 1.0) },
            CubeVert { pos: vec3(-1.0,  1.0,  1.0), color: vec4(0.5, 1.0, 0.5, 1.0), uv: vec2(0.0, 1.0) },

            CubeVert { pos: vec3(-1.0, -1.0, -1.0), color: vec4(0.5, 0.5, 1.0, 1.0), uv: vec2(0.0, 0.0) },
            CubeVert { pos: vec3(-1.0,  1.0, -1.0), color: vec4(0.5, 0.5, 1.0, 1.0), uv: vec2(1.0, 0.0) },
            CubeVert { pos: vec3(-1.0,  1.0,  1.0), color: vec4(0.5, 0.5, 1.0, 1.0), uv: vec2(1.0, 1.0) },
            CubeVert { pos: vec3(-1.0, -1.0,  1.0), color: vec4(0.5, 0.5, 1.0, 1.0), uv: vec2(0.0, 1.0) },

            CubeVert { pos: vec3(1.0, -1.0, -1.0), color: vec4(1.0, 0.5, 0.0, 1.0), uv: vec2(0.0, 0.0) },
            CubeVert { pos: vec3(1.0,  1.0, -1.0), color: vec4(1.0, 0.5, 0.0, 1.0), uv: vec2(1.0, 0.0) },
            CubeVert { pos: vec3(1.0,  1.0,  1.0), color: vec4 (1.0, 0.5, 0.0, 1.0), uv: vec2(1.0, 1.0) },
            CubeVert { pos: vec3(1.0, -1.0,  1.0), color: vec4(1.0, 0.5, 0.0, 1.0), uv: vec2(0.0, 1.0) },

            CubeVert { pos: vec3(-1.0, -1.0, -1.0), color: vec4(0.0, 0.5, 1.0, 1.0), uv: vec2(0.0, 0.0) },
            CubeVert { pos: vec3(-1.0, -1.0,  1.0), color: vec4(0.0, 0.5, 1.0, 1.0), uv: vec2(1.0, 0.0) },
            CubeVert { pos: vec3(1.0, -1.0,  1.0), color: vec4(0.0, 0.5, 1.0, 1.0), uv: vec2(1.0, 1.0) },
            CubeVert { pos: vec3(1.0, -1.0, -1.0), color: vec4(0.0, 0.5, 1.0, 1.0), uv: vec2(0.0, 1.0) },

            CubeVert { pos: vec3(-1.0,  1.0, -1.0), color: vec4(1.0, 0.0, 0.5, 1.0), uv: vec2(0.0, 0.0) },
            CubeVert { pos: vec3(-1.0,  1.0,  1.0), color: vec4(1.0, 0.0, 0.5, 1.0), uv: vec2(1.0, 0.0) },
            CubeVert { pos: vec3(1.0,  1.0,  1.0),  color: vec4(1.0, 0.0, 0.5, 1.0), uv: vec2(1.0, 1.0) },
            CubeVert { pos: vec3(1.0,  1.0, -1.0),  color: vec4(1.0, 0.0, 0.5, 1.0), uv: vec2(0.0, 1.0) },
        ];
        let vertices_cube = Buffer::new(ctx.clone(), BufferUsage::Immutable, vertices_cube);

        #[rustfmt::skip]
        let indicies_cube = &[
            0, 1, 2,  0, 2, 3,
            6, 5, 4,  7, 6, 4,
            8, 9, 10,  8, 10, 11,
            14, 13, 12,  15, 14, 12,
            16, 17, 18,  16, 18, 19,
            22, 21, 20,  23, 22, 20
        ];
        let indicies_cube = IndexBuffer::new(ctx.clone(), BufferUsage::Immutable, indicies_cube);

        let display_pipeline = Pipeline::new(
            ctx.clone(),
            match ctx.info().backend {
                Backend::OpenGl => ShaderSource::Glsl {
                    vertex: display_shader::VERTEX,
                    fragment: display_shader::FRAGMENT,
                },
                Backend::Metal => ShaderSource::Msl {
                    program: display_shader::METAL,
                },
            },
            display_shader::meta(),
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        )
        .unwrap();

        let offscreen_pipeline = Pipeline::new(
            ctx.clone(),
            match ctx.info().backend {
                Backend::OpenGl => ShaderSource::Glsl {
                    vertex: offscreen_shader::VERTEX,
                    fragment: offscreen_shader::FRAGMENT,
                },
                Backend::Metal => ShaderSource::Msl {
                    program: offscreen_shader::METAL,
                },
            },
            offscreen_shader::meta(),
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        )
        .unwrap();

        Stage {
            vertices_cube,
            indicies_cube,
            display_pipeline,
            offscreen_pipeline,
            offscreen_pass,
            rx: 0.,
            ry: 0.,
            ctx,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        let (width, height) = window::screen_size();
        let proj = Mat4::perspective_rh_gl(60.0f32.to_radians(), width / height, 0.01, 10.0);
        let view = Mat4::look_at_rh(
            vec3(0.0, 1.5, 3.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );
        let view_proj = proj * view;

        self.rx += 0.01;
        self.ry += 0.03;
        let model = Mat4::from_rotation_y(self.ry) * Mat4::from_rotation_x(self.rx);

        let vs_params = display_shader::Uniforms {
            mvp: view_proj * model,
        };

        // the offscreen pass, rendering a rotating, untextured cube into a render target image
        self.offscreen_pass
            .perform(PassAction::clear_color(1.0, 1.0, 1.0, 1.0), || {
                DrawCall {
                    pipeline: &self.offscreen_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffers: &bind_buffers![
                        (&self.vertices_cube) as <CubeVert>::pos,
                        (&self.vertices_cube) as <CubeVert>::color,
                    ],
                    index_buffer: &self.indicies_cube,
                    textures: &[],
                    uniform_data: bytemuck::bytes_of(&vs_params),
                }
                .execute();
            });

        // and the display-pass, rendering a rotating, textured cube, using the
        // previously rendered offscreen render-target as texture
        self.ctx
            .perform_default_render_pass(PassAction::clear_color(0.0, 0., 0.45, 1.), || {
                DrawCall {
                    pipeline: &self.display_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffers: &bind_buffers![
                        (&self.vertices_cube) as <CubeVert>::pos,
                        (&self.vertices_cube) as <CubeVert>::color,
                        (&self.vertices_cube) as <CubeVert>::uv,
                    ],
                    index_buffer: &self.indicies_cube,
                    textures: &[&self.offscreen_pass.color_attachments()[0]],
                    uniform_data: bytemuck::bytes_of(&vs_params),
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

mod display_shader {
    use bytemuck::{Pod, Zeroable};
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec4 in_pos;
    attribute vec4 in_color;
    attribute vec2 in_uv;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * in_pos;
        color = in_color;
        uv = in_uv;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = color * texture2D(tex, uv);
    }
    "#;

    pub const METAL: &str = r#"#include <metal_stdlib>
    using namespace metal;

    struct Uniforms
    {
        float4x4 mvp;
    };

    struct Vertex
    {
        float3 in_pos      [[attribute(0)]];
        float4 in_color    [[attribute(1)]];
        float2 in_uv       [[attribute(2)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
        float2 uv [[user(locn1)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = uniforms.mvp * float4(v.in_pos, 1.0);
        out.color = v.in_color;
        out.uv = v.in_uv;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return in.color * tex.sample(texSmplr, in.uv);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4)],
            attributes: vec![
                VertexAttribute::new("in_pos", VertexFormat::Float3),
                VertexAttribute::new("in_color", VertexFormat::Float4),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
        }
    }

    #[repr(C)]
    #[derive(Zeroable, Pod, Clone, Copy)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}

mod offscreen_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 in_pos;
    attribute vec4 in_color;

    varying lowp vec4 color;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * vec4(in_pos, 1.0);
        color = in_color;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;

    void main() {
        gl_FragColor = color;
    }
    "#;

    pub const METAL: &str = r#"#include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float4x4 mvp;
    };

    struct Vertex
    {
        float3 in_pos      [[attribute(0)]];
        float4 in_color    [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = uniforms.mvp * float4(v.in_pos, 1.0) * float4(1.0, -1.0, 1.0, 1.0);
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
            uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4)],
            attributes: vec![
                VertexAttribute::new("in_pos", VertexFormat::Float3),
                VertexAttribute::new("in_color", VertexFormat::Float4),
            ],
        }
    }
}
