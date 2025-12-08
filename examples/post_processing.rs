use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use miniquad::*;
///! A post processing example. Draws a rotating cube with
///! differently colored sides. Should look like this:
///! https://youtu.be/hdWWe-TkkfM
use std::rc::Rc;
use winit::{event::WindowEvent, window::Window};

fn main() {
    miniquad::run::<Stage>(Conf::default());
}

struct Stage {
    vertices_cube: VertexBuffer<CubeVert>,
    indicies_cube: IndexBuffer,
    vertices_quad: VertexBuffer<QuadVert>,
    indicies_quad: IndexBuffer,
    post_processing_pipeline: Pipeline<post_processing_shader::Uniforms>,
    offscreen_pipeline: Pipeline<offscreen_shader::Uniforms>,
    offscreen_pass: RenderPass,
    rx: f32,
    ry: f32,

    ctx: Rc<GlContext>,
}

impl EventHandler for Stage {
    fn update(&mut self) {
        self.rx += 0.01;
        self.ry += 0.03;
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(sz) => self.resize_event(sz.into()),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: FsServerHandle) -> Stage {
        let (width, height) = ctx.screen_size();
        let color_img = ctx.new_empty_texture(
            width,
            height,
            TextureParams {
                internal_format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = ctx.new_empty_texture(
            width,
            height,
            TextureParams {
                internal_format: TextureFormat::DepthU16,
                ..Default::default()
            },
        );

        let offscreen_pass = ctx.new_render_pass(vec![color_img], Some(depth_img));

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
        let vertices_cube = VertexBuffer::new(ctx.clone(), BufferUsage::Immutable, vertices_cube);

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

        #[rustfmt::skip]
        let vertices_quad = &[
            QuadVert { pos: vec2(-1.0, -1.0), uv: vec2(0.0, 0.0) },
            QuadVert { pos: vec2(1.0, -1.0), uv: vec2(1.0, 0.0) },
            QuadVert { pos: vec2(1.0,  1.0), uv: vec2(1.0, 1.0) },
            QuadVert { pos: vec2(-1.0,  1.0), uv: vec2(0.0, 1.0) },
        ];
        let vertices_quad = ctx.new_vertex_buffer(BufferUsage::Immutable, vertices_quad);

        let indicies_quad = &[0, 1, 2, 0, 2, 3];
        let indicies_quad = ctx.new_index_buffer(BufferUsage::Immutable, indicies_quad);

        let post_processing_pipeline = ctx
            .new_pipeline(
                post_processing_shader::VERTEX,
                post_processing_shader::FRAGMENT,
                PipelineParams::default(),
                [
                    VertexAttribute::new("pos", VertexFormat::F32x2),
                    VertexAttribute::new("uv", VertexFormat::F32x2),
                ],
                [UniformDesc::new_scalar("resolution", UniformType::F32x2)],
                ["tex"],
            )
            .unwrap();

        let offscreen_pipeline = ctx
            .new_pipeline::<&'static str, _>(
                offscreen_shader::VERTEX,
                offscreen_shader::FRAGMENT,
                PipelineParams {
                    depth_test: Some(Comparison::LessOrEqual),
                    ..Default::default()
                },
                [
                    VertexAttribute::new("pos", VertexFormat::F32x3),
                    VertexAttribute::new("color0", VertexFormat::F32x4),
                ],
                [UniformDesc::new_scalar("mvp", UniformType::F32x4x4)],
                [],
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

impl Stage {
    fn resize_event(&mut self, (width, height): (u32, u32)) {
        let color_img = self.ctx.new_empty_texture(
            width,
            height,
            TextureParams {
                internal_format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = self.ctx.new_empty_texture(
            width,
            height,
            TextureParams {
                internal_format: TextureFormat::DepthU16,
                ..Default::default()
            },
        );

        self.offscreen_pass = RenderPass::new(self.ctx.clone(), vec![color_img], Some(depth_img));
    }

    fn draw(&mut self) {
        let (width, height) = self.ctx.screen_size();
        let (width, height) = (width as f32, height as f32);
        let proj = Mat4::perspective_rh_gl(60.0f32.to_radians(), width / height, 0.01, 10.0);
        let view = Mat4::look_at_rh(
            vec3(0.0, 1.5, 3.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );
        let view_proj = proj * view;

        let model = Mat4::from_rotation_y(self.ry) * Mat4::from_rotation_y(self.rx);

        // the offscreen pass, rendering an rotating, untextured cube into a render target image
        self.offscreen_pass
            .perform(PassAction::clear_depth_color(1.0, 1.0, 1.0, 1.0), |_, _| {
                self.ctx.submit_drawcall(DrawCall {
                    pipeline: &self.offscreen_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffers: &bind_vertex_buffers![
                        (&self.vertices_cube) as <CubeVert>::pos,
                        (&self.vertices_cube) as <CubeVert>::color,
                    ],
                    index_buffer: self.indicies_cube.bind(),
                    textures: &[],
                    uniforms: &offscreen_shader::Uniforms {
                        mvp: view_proj * model,
                    },
                });
            });

        // and the post-processing-pass, rendering a rotating, textured cube, using the
        // previously rendered offscreen render-target as texture
        self.ctx.perform_default_render_pass(
            PassAction::clear_depth_color(1.0, 1.0, 1.0, 1.0),
            |_, _| {
                self.ctx.submit_drawcall(DrawCall {
                    pipeline: &self.post_processing_pipeline,
                    base_element: 0,
                    num_elements: 6,
                    vertex_buffers: &bind_vertex_buffers![
                        (&self.vertices_quad) as <QuadVert>::pos,
                        (&self.vertices_quad) as <QuadVert>::uv,
                    ],
                    index_buffer: self.indicies_quad.bind(),
                    textures: &[self.offscreen_pass.color_attachments()[0].bind()],
                    uniforms: &post_processing_shader::Uniforms {
                        resolution: vec2(width, height),
                    },
                });
            },
        );
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct CubeVert {
    pub pos: Vec3,
    pub color: Vec4,
    pub uv: Vec2,
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct QuadVert {
    pub pos: Vec2,
    pub uv: Vec2,
}

mod post_processing_shader {
    use bytemuck::{Pod, Zeroable};

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

    #[repr(C)]
    #[derive(Pod, Zeroable, Clone, Copy)]
    pub struct Uniforms {
        pub resolution: glam::Vec2,
    }
}

mod offscreen_shader {
    use bytemuck::{Pod, Zeroable};

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

    #[repr(C)]
    #[derive(Pod, Zeroable, Clone, Copy)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}
