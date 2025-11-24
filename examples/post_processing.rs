use std::rc::Rc;

///! A post processing example. Draws a rotating cube with
///! differently colored sides. Should look like this:
///! https://youtu.be/hdWWe-TkkfM
use miniquad::*;

use glam::{vec2, vec3, Mat4};

struct Stage {
    vertices_cube: Buffer,
    indicies_cube: Buffer,
    vertices_quad: Buffer,
    indicies_quad: Buffer,
    post_processing_pipeline: Pipeline,
    offscreen_pipeline: Pipeline,
    offscreen_pass: RenderPass,
    rx: f32,
    ry: f32,

    ctx: Rc<GlContext>,
}

impl Stage {
    pub fn new() -> Stage {
        let ctx = window::new_rendering_backend();
        let (w, h) = window::screen_size();
        let color_img = Texture::new(
            ctx.clone(),
            TextureSource::Empty,
            TextureParams {
                width: w as _,
                height: h as _,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = Texture::new(
            ctx.clone(),
            TextureSource::Empty,
            TextureParams {
                width: w as _,
                height: h as _,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        );

        let offscreen_pass = RenderPass::new(ctx.clone(), vec![color_img], None, Some(depth_img));

        #[rustfmt::skip]
        let vertices_cube: &[f32] = &[
            /* pos               color                   uvs */
            -1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     0.0, 0.0,
             1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     1.0, 1.0,
            -1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     0.0, 1.0,

            -1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     0.0, 0.0,
             1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     1.0, 1.0,
            -1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     0.0, 1.0,

            -1.0, -1.0, -1.0,    0.5, 0.5, 1.0, 1.0,     0.0, 0.0,
            -1.0,  1.0, -1.0,    0.5, 0.5, 1.0, 1.0,     1.0, 0.0,
            -1.0,  1.0,  1.0,    0.5, 0.5, 1.0, 1.0,     1.0, 1.0,
            -1.0, -1.0,  1.0,    0.5, 0.5, 1.0, 1.0,     0.0, 1.0,

             1.0, -1.0, -1.0,    1.0, 0.5, 0.0, 1.0,     0.0, 0.0,
             1.0,  1.0, -1.0,    1.0, 0.5, 0.0, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    1.0, 0.5, 0.0, 1.0,     1.0, 1.0,
             1.0, -1.0,  1.0,    1.0, 0.5, 0.0, 1.0,     0.0, 1.0,

            -1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,     0.0, 0.0,
            -1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,     1.0, 0.0,
             1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,     1.0, 1.0,
             1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,     0.0, 1.0,

            -1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,     0.0, 0.0,
            -1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,     1.0, 1.0,
             1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,     0.0, 1.0
        ];
        let vertices_cube = Buffer::new(
            ctx.clone(),
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices_cube),
        );

        #[rustfmt::skip]
        let indicies_cube: &[u16] = &[
            0, 1, 2,  0, 2, 3,
            6, 5, 4,  7, 6, 4,
            8, 9, 10,  8, 10, 11,
            14, 13, 12,  15, 14, 12,
            16, 17, 18,  16, 18, 19,
            22, 21, 20,  23, 22, 20
        ];
        let indicies_cube = Buffer::new(
            ctx.clone(),
            BufferType::IndexBuffer(IndexBufferElementSize::Two),
            BufferUsage::Immutable,
            BufferSource::slice(&indicies_cube),
        );

        #[rustfmt::skip]
        let vertices: &[f32] = &[
            /* pos         uvs */
            -1.0, -1.0,    0.0, 0.0,
             1.0, -1.0,    1.0, 0.0,
             1.0,  1.0,    1.0, 1.0,
            -1.0,  1.0,    0.0, 1.0,
        ];
        let vertices_quad = Buffer::new(
            ctx.clone(),
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indicies_quad: &[u16] = &[0, 1, 2, 0, 2, 3];
        let indicies_quad = Buffer::new(
            ctx.clone(),
            BufferType::IndexBuffer(IndexBufferElementSize::Two),
            BufferUsage::Immutable,
            BufferSource::slice(&indicies_quad),
        );

        let post_processing_pipeline = Pipeline::new(
            ctx.clone(),
            match ctx.info().backend {
                Backend::OpenGl => ShaderSource::Glsl {
                    vertex: post_processing_shader::VERTEX,
                    fragment: post_processing_shader::FRAGMENT,
                },
                Backend::Metal => unimplemented!(),
            },
            post_processing_shader::meta(),
            PipelineParams::default(),
        )
        .unwrap();

        let offscreen_pipeline = Pipeline::new(
            ctx.clone(),
            ShaderSource::Glsl {
                vertex: offscreen_shader::VERTEX,
                fragment: offscreen_shader::FRAGMENT,
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
            vertices_quad,
            indicies_quad,
            post_processing_pipeline,
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

    fn resize_event(&mut self, width: f32, height: f32) {
        let color_img = Texture::new(
            self.ctx.clone(),
            TextureSource::Empty,
            TextureParams {
                width: width as _,
                height: height as _,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = Texture::new(
            self.ctx.clone(),
            TextureSource::Empty,
            TextureParams {
                width: width as _,
                height: height as _,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        );

        self.offscreen_pass =
            RenderPass::new(self.ctx.clone(), vec![color_img], None, Some(depth_img));
    }

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
        let model = Mat4::from_rotation_y(self.ry) * Mat4::from_rotation_y(self.rx);
        let uniforms = offscreen_shader::Uniforms {
            mvp: view_proj * model,
        };

        // the offscreen pass, rendering an rotating, untextured cube into a render target image
        self.offscreen_pass
            .perform(PassAction::clear_color(1.0, 1.0, 1.0, 1.0), || {
                DrawCall {
                    pipeline: &self.offscreen_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffers: &[
                        self.vertices_cube.binding(0, 36),
                        self.vertices_cube.binding(12, 36),
                    ],
                    index_buffer: &self.indicies_cube,
                    textures: &[],
                    uniform_data: bytemuck::bytes_of(&uniforms),
                }
                .execute()
            });

        let (w, h) = window::screen_size();
        let uniforms = post_processing_shader::Uniforms {
            resolution: vec2(w, h),
        };
        // and the post-processing-pass, rendering a rotating, textured cube, using the
        // previously rendered offscreen render-target as texture
        self.ctx
            .perform_default_render_pass(PassAction::clear_color(1.0, 1.0, 1.0, 1.0), || {
                DrawCall {
                    pipeline: &self.post_processing_pipeline,
                    base_element: 0,
                    num_elements: 6,
                    vertex_buffers: &[
                        self.vertices_quad.binding(0, 16),
                        self.vertices_quad.binding(8, 16),
                    ],
                    index_buffer: &self.indicies_quad,
                    textures: &[&self.offscreen_pass.color_attachments()[0]],
                    uniform_data: bytemuck::bytes_of(&uniforms),
                }
                .execute()
            });
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), || Box::new(Stage::new()));
}

mod post_processing_shader {
    use bytemuck::{Pod, Zeroable};
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(pos, 0, 1);
        texcoord = uv;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    precision lowp float;

    varying vec2 texcoord;

    uniform sampler2D tex;
    uniform vec2 resolution;



    // Source: https://github.com/Jam3/glsl-fast-gaussian-blur/blob/master/5.glsl
    vec4 blur5(sampler2D image, vec2 uv, vec2 resolution, vec2 direction) {
        vec4 color = vec4(0.0);
        vec2 off1 = vec2(1.3333333333333333) * direction;
        color += texture2D(image, uv) * 0.29411764705882354;
        color += texture2D(image, uv + (off1 / resolution)) * 0.35294117647058826;
        color += texture2D(image, uv - (off1 / resolution)) * 0.35294117647058826;
        return color;
    }

    void main() {
        gl_FragColor = blur5(tex, texcoord, resolution, vec2(3.0));
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: vec![UniformDesc::new("resolution", UniformType::Float2)],
            attributes: vec![
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
        }
    }

    #[repr(C)]
    #[derive(Pod, Zeroable, Clone, Copy)]
    pub struct Uniforms {
        pub resolution: glam::Vec2,
    }
}

mod offscreen_shader {
    use bytemuck::{Pod, Zeroable};
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec4 pos;
    attribute vec4 color0;

    varying lowp vec4 color;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * pos;
        color = color0;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100

    varying lowp vec4 color;

    void main() {
        gl_FragColor = color;
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4)],
            attributes: vec![
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("color0", VertexFormat::Float4),
            ],
        }
    }

    #[repr(C)]
    #[derive(Pod, Zeroable, Clone, Copy)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}
