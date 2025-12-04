use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use miniquad::*;
///! An offscreen render example. Draws a cube that has
///! images of rotating cubes on each side. Should look like this:
///! https://youtu.be/isKW3nQ-jW4
use std::rc::Rc;
use winit::{event::WindowEvent, window::Window};

fn main() {
    miniquad::run::<Stage>(Conf::default());
}

struct Stage {
    vertices_cube: VertexBuffer<CubeVert>,
    indicies_cube: IndexBuffer,
    display_pipeline: Pipeline,
    offscreen_pipeline: Pipeline,
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
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: FsServerHandle) -> Stage {
        let color_img = ctx.new_texture(
            TextureSource::Empty,
            TextureParams {
                width: 256,
                height: 256,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = ctx.new_texture(
            TextureSource::Empty,
            TextureParams {
                width: 256,
                height: 256,
                format: TextureFormat::DepthU16,
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
        let indicies_cube = ctx.new_index_buffer(BufferUsage::Immutable, indicies_cube);

        let display_pipeline = ctx
            .new_pipeline(
                display_shader::VERTEX,
                display_shader::FRAGMENT,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    ..Default::default()
                },
                [
                    VertexAttribute::new("in_pos", VertexFormat::F32x3),
                    VertexAttribute::new("in_color", VertexFormat::F32x4),
                    VertexAttribute::new("in_uv", VertexFormat::F32x2),
                ],
                [UniformDesc::new_scalar("mvp", UniformType::F32x4x4)],
                ["tex"],
            )
            .unwrap();

        let offscreen_pipeline = ctx
            .new_pipeline::<&'static str>(
                offscreen_shader::VERTEX,
                offscreen_shader::FRAGMENT,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    ..Default::default()
                },
                [
                    VertexAttribute::new("in_pos", VertexFormat::F32x3),
                    VertexAttribute::new("in_color", VertexFormat::F32x4),
                ],
                [UniformDesc::new_scalar("mvp", UniformType::F32x4x4)],
                [],
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

impl Stage {
    pub fn draw(&mut self) {
        let (width, height) = self.ctx.screen_size();
        let (width, height) = (width as f32, height as f32);
        let proj = Mat4::perspective_rh_gl(60.0f32.to_radians(), width / height, 0.01, 10.0);
        let view = Mat4::look_at_rh(
            vec3(0.0, 1.5, 3.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );
        let view_proj = proj * view;

        let model = Mat4::from_rotation_y(self.ry) * Mat4::from_rotation_x(self.rx);

        let vs_params = display_shader::Uniforms {
            mvp: view_proj * model,
        };

        // the offscreen pass, rendering a rotating, untextured cube into a render target image
        self.offscreen_pass
            .perform(PassAction::clear_depth_color(1.0, 1.0, 1.0, 1.0), |_, _| {
                DrawCall {
                    ctx: &self.ctx,
                    pipeline: &self.offscreen_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffers: &bind_vertex_buffers![
                        (&self.vertices_cube) as <CubeVert>::pos,
                        (&self.vertices_cube) as <CubeVert>::color,
                    ],
                    index_buffer: self.indicies_cube.bind(),
                    textures: &[],
                    uniform_data: bytemuck::bytes_of(&vs_params),
                }
                .execute();
            });

        // and the display-pass, rendering a rotating, textured cube, using the
        // previously rendered offscreen render-target as texture
        self.ctx.perform_default_render_pass(
            PassAction::clear_depth_color(0.0, 0., 0.45, 1.),
            |_, _| {
                DrawCall {
                    ctx: &self.ctx,
                    pipeline: &self.display_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffers: &bind_vertex_buffers![
                        (&self.vertices_cube) as <CubeVert>::pos,
                        (&self.vertices_cube) as <CubeVert>::color,
                        (&self.vertices_cube) as <CubeVert>::uv,
                    ],
                    index_buffer: self.indicies_cube.bind(),
                    textures: &[self.offscreen_pass.color_attachments()[0].bind()],
                    uniform_data: bytemuck::bytes_of(&vs_params),
                }
                .execute();
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

mod display_shader {
    use bytemuck::{Pod, Zeroable};

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

    #[repr(C)]
    #[derive(Zeroable, Pod, Clone, Copy)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}

mod offscreen_shader {
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
}
