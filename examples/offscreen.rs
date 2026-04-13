//! An offscreen render example. Draws a cube that has
//! images of rotating cubes on each side. Should look like this:
//! https://youtu.be/isKW3nQ-jW4

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use mimiq::*;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    vertices_display_cube: VertexBuffer<CubeDisplayVert>,
    vertices_cube: VertexBuffer<CubeVert>,
    indicies_cube: IndexBuffer,
    display_pipeline: Pipeline<DisplayMeta>,
    offscreen_pipeline: Pipeline<OffscreenMeta>,
    offscreen_pass: RenderPass,
    rx: f32,
    ry: f32,
    ctx: Rc<GlContext>,
}

impl EventHandler<()> for App {
    fn update(&mut self, dt: Duration) {
        self.rx += 0.6 * dt.as_secs_f32();
        self.ry += 1.8 * dt.as_secs_f32();
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: Rc<dyn FsServer>, _init: ()) -> App {
        let color_img = ctx.new_empty_texture(
            256,
            256,
            Texture2DParams { internal_format: Texture2DFormat::RGBA8, ..Default::default() },
        );
        let depth_img = ctx.new_empty_texture(
            256,
            256,
            Texture2DParams { internal_format: Texture2DFormat::DepthU16, ..Default::default() },
        );
        let offscreen_pass = ctx.new_render_pass(vec![color_img], Some(depth_img));

        #[rustfmt::skip]
        let vertices_display_cube = [
            CubeDisplayVert { pos: vec3(-1.0, -1.0, -1.0), color: vec4(1.0, 0.5, 0.5, 1.0), uv: vec2(0.0, 0.0) },
            CubeDisplayVert { pos: vec3(1.0, -1.0, -1.0), color: vec4(1.0, 0.5, 0.5, 1.0),  uv: vec2(1.0, 0.0) },
            CubeDisplayVert { pos: vec3(1.0,  1.0, -1.0), color: vec4(1.0, 0.5, 0.5, 1.0), uv: vec2(1.0, 1.0) },
            CubeDisplayVert { pos: vec3(-1.0,  1.0, -1.0),  color: vec4(1.0, 0.5, 0.5, 1.0), uv: vec2(0.0, 1.0) },

            CubeDisplayVert { pos: vec3(-1.0, -1.0,  1.0), color: vec4(0.5, 1.0, 0.5, 1.0), uv: vec2(0.0, 0.0) },
            CubeDisplayVert { pos: vec3(1.0, -1.0,  1.0),  color: vec4(0.5, 1.0, 0.5, 1.0), uv: vec2(1.0, 0.0) },
            CubeDisplayVert { pos: vec3(1.0,  1.0,  1.0),  color: vec4(0.5, 1.0, 0.5, 1.0),  uv: vec2(1.0, 1.0) },
            CubeDisplayVert { pos: vec3(-1.0,  1.0,  1.0), color: vec4(0.5, 1.0, 0.5, 1.0), uv: vec2(0.0, 1.0) },

            CubeDisplayVert { pos: vec3(-1.0, -1.0, -1.0), color: vec4(0.5, 0.5, 1.0, 1.0), uv: vec2(0.0, 0.0) },
            CubeDisplayVert { pos: vec3(-1.0,  1.0, -1.0), color: vec4(0.5, 0.5, 1.0, 1.0), uv: vec2(1.0, 0.0) },
            CubeDisplayVert { pos: vec3(-1.0,  1.0,  1.0), color: vec4(0.5, 0.5, 1.0, 1.0), uv: vec2(1.0, 1.0) },
            CubeDisplayVert { pos: vec3(-1.0, -1.0,  1.0), color: vec4(0.5, 0.5, 1.0, 1.0), uv: vec2(0.0, 1.0) },

            CubeDisplayVert { pos: vec3(1.0, -1.0, -1.0), color: vec4(1.0, 0.5, 0.0, 1.0), uv: vec2(0.0, 0.0) },
            CubeDisplayVert { pos: vec3(1.0,  1.0, -1.0), color: vec4(1.0, 0.5, 0.0, 1.0), uv: vec2(1.0, 0.0) },
            CubeDisplayVert { pos: vec3(1.0,  1.0,  1.0), color: vec4 (1.0, 0.5, 0.0, 1.0), uv: vec2(1.0, 1.0) },
            CubeDisplayVert { pos: vec3(1.0, -1.0,  1.0), color: vec4(1.0, 0.5, 0.0, 1.0), uv: vec2(0.0, 1.0) },

            CubeDisplayVert { pos: vec3(-1.0, -1.0, -1.0), color: vec4(0.0, 0.5, 1.0, 1.0), uv: vec2(0.0, 0.0) },
            CubeDisplayVert { pos: vec3(-1.0, -1.0,  1.0), color: vec4(0.0, 0.5, 1.0, 1.0), uv: vec2(1.0, 0.0) },
            CubeDisplayVert { pos: vec3(1.0, -1.0,  1.0), color: vec4(0.0, 0.5, 1.0, 1.0), uv: vec2(1.0, 1.0) },
            CubeDisplayVert { pos: vec3(1.0, -1.0, -1.0), color: vec4(0.0, 0.5, 1.0, 1.0), uv: vec2(0.0, 1.0) },

            CubeDisplayVert { pos: vec3(-1.0,  1.0, -1.0), color: vec4(1.0, 0.0, 0.5, 1.0), uv: vec2(0.0, 0.0) },
            CubeDisplayVert { pos: vec3(-1.0,  1.0,  1.0), color: vec4(1.0, 0.0, 0.5, 1.0), uv: vec2(1.0, 0.0) },
            CubeDisplayVert { pos: vec3(1.0,  1.0,  1.0),  color: vec4(1.0, 0.0, 0.5, 1.0), uv: vec2(1.0, 1.0) },
            CubeDisplayVert { pos: vec3(1.0,  1.0, -1.0),  color: vec4(1.0, 0.0, 0.5, 1.0), uv: vec2(0.0, 1.0) },
        ];
        let vertices_cube = vertices_display_cube
            .into_iter()
            .map(|v| CubeVert { pos: v.pos, color: v.color })
            .collect::<Vec<_>>();

        let vertices_display_cube =
            ctx.new_vertex_buffer(BufferUsage::Immutable, &vertices_display_cube);
        let vertices_cube = ctx.new_vertex_buffer(BufferUsage::Immutable, &vertices_cube);

        #[rustfmt::skip]
        let indicies_cube = ctx.new_index_buffer(BufferUsage::Immutable, &[
            0, 1, 2,  0, 2, 3,
            6, 5, 4,  7, 6, 4,
            8, 9, 10,  8, 10, 11,
            14, 13, 12,  15, 14, 12,
            16, 17, 18,  16, 18, 19,
            22, 21, 20,  23, 22, 20
        ]);

        let display_pipeline = ctx.new_pipeline();
        let offscreen_pipeline = ctx.new_pipeline();

        App {
            vertices_cube,
            vertices_display_cube,
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

impl App {
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

        // the offscreen pass, rendering a rotating, untextured cube into a render target image
        self.offscreen_pass
            .pass(Clear::depth_color(Color::WHITE), |_, _| {
                self.ctx.draw(DrawCall {
                    pipeline: &self.offscreen_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffer: &self.vertices_cube,
                    index_buffer: &self.indicies_cube,
                    images: &NoImages,
                    uniforms: &Uniforms { mvp: view_proj * model },
                });
            });

        // and the display-pass, rendering a rotating, textured cube, using the
        // previously rendered offscreen render-target as texture
        self.ctx
            .default_pass(Clear::depth_color(Color::DARKBLUE), |_, _| {
                self.ctx.draw(DrawCall {
                    pipeline: &self.display_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffer: &self.vertices_display_cube,
                    index_buffer: &self.indicies_cube,
                    images: &self.offscreen_pass.color_attachments()[0],
                    uniforms: &Uniforms { mvp: view_proj * model },
                });
            });
    }
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy, Vertex)]
pub struct CubeVert {
    pub pos: Vec3,
    pub color: Vec4,
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy, Vertex)]
pub struct CubeDisplayVert {
    pub pos: Vec3,
    pub color: Vec4,
    pub uv: Vec2,
}

pub struct DisplayMeta;

impl PipelineMeta for DisplayMeta {
    const VERTEX_SHADER: &str = include_str!("shaders/mvp_color_texture.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/color_texture.frag");

    const IMAGES_NAMES: &str = "tex";
    type Images = Texture2D;
    type Vertex = CubeDisplayVert;
    type Uniforms = Uniforms;
    const PARAMS: PipelineParams =
        PipelineParams { depth_test: Some(Comparison::LessOrEqual), ..default_pipeline_params() };
}

pub struct OffscreenMeta;

impl PipelineMeta for OffscreenMeta {
    const VERTEX_SHADER: &str = include_str!("shaders/mvp_color.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/basic_color.frag");

    const IMAGES_NAMES: () = ();
    type Images = NoImages;
    type Vertex = CubeVert;
    type Uniforms = Uniforms;
    const PARAMS: PipelineParams =
        PipelineParams { depth_test: Some(Comparison::LessOrEqual), ..default_pipeline_params() };
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy, UniformBlock)]
pub struct Uniforms {
    pub mvp: glam::Mat4,
}
