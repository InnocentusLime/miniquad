//! A post processing example. Draws a rotating cube with
//! differently colored sides. Should look like this:
//! https://youtu.be/hdWWe-TkkfM

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4, vec3, vec4};
use mimiq::*;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    vertices_cube: VertexBuffer<CubeVert>,
    indicies_cube: IndexBuffer,
    offscreen_pipeline: Pipeline<OffscreenMeta>,
    offscreen_pass: RenderPass,
    rx: f32,
    ry: f32,

    ctx: Rc<GlContext>,
}

impl EventHandler<()> for App {
    fn update(&mut self, dt: Duration) {
        self.rx += dt.as_secs_f32() * 0.6;
        self.ry += dt.as_secs_f32() * 1.8;
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(sz) => self.resize_event(sz.into()),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: Rc<dyn FsServer>, _init: ()) -> App {
        let (width, height) = ctx.screen_size();
        let color_img = ctx.new_empty_texture(
            width,
            height,
            Texture2DParams { internal_format: Texture2DFormat::RGBA8, ..Default::default() },
        );
        let depth_img = ctx.new_empty_texture(
            width,
            height,
            Texture2DParams { internal_format: Texture2DFormat::DepthU16, ..Default::default() },
        );

        let offscreen_pass = ctx.new_render_pass(vec![color_img], Some(depth_img));

        #[rustfmt::skip]
        let vertices_cube = ctx.new_vertex_buffer(BufferUsage::Immutable, &[
            CubeVert { v_pos: vec3(-1.0, -1.0, -1.0), v_color: vec4(1.0, 0.5, 0.5, 1.0) },
            CubeVert { v_pos: vec3(1.0, -1.0, -1.0), v_color: vec4(1.0, 0.5, 0.5, 1.0) },
            CubeVert { v_pos: vec3(1.0,  1.0, -1.0), v_color: vec4(1.0, 0.5, 0.5, 1.0) },
            CubeVert { v_pos: vec3(-1.0,  1.0, -1.0),  v_color: vec4(1.0, 0.5, 0.5, 1.0) },

            CubeVert { v_pos: vec3(-1.0, -1.0,  1.0), v_color: vec4(0.5, 1.0, 0.5, 1.0) },
            CubeVert { v_pos: vec3(1.0, -1.0,  1.0),  v_color: vec4(0.5, 1.0, 0.5, 1.0) },
            CubeVert { v_pos: vec3(1.0,  1.0,  1.0),  v_color: vec4(0.5, 1.0, 0.5, 1.0) },
            CubeVert { v_pos: vec3(-1.0,  1.0,  1.0), v_color: vec4(0.5, 1.0, 0.5, 1.0) },

            CubeVert { v_pos: vec3(-1.0, -1.0, -1.0), v_color: vec4(0.5, 0.5, 1.0, 1.0) },
            CubeVert { v_pos: vec3(-1.0,  1.0, -1.0), v_color: vec4(0.5, 0.5, 1.0, 1.0) },
            CubeVert { v_pos: vec3(-1.0,  1.0,  1.0), v_color: vec4(0.5, 0.5, 1.0, 1.0) },
            CubeVert { v_pos: vec3(-1.0, -1.0,  1.0), v_color: vec4(0.5, 0.5, 1.0, 1.0) },

            CubeVert { v_pos: vec3(1.0, -1.0, -1.0), v_color: vec4(1.0, 0.5, 0.0, 1.0) },
            CubeVert { v_pos: vec3(1.0,  1.0, -1.0), v_color: vec4(1.0, 0.5, 0.0, 1.0) },
            CubeVert { v_pos: vec3(1.0,  1.0,  1.0), v_color: vec4 (1.0, 0.5, 0.0, 1.0) },
            CubeVert { v_pos: vec3(1.0, -1.0,  1.0), v_color: vec4(1.0, 0.5, 0.0, 1.0) },

            CubeVert { v_pos: vec3(-1.0, -1.0, -1.0), v_color: vec4(0.0, 0.5, 1.0, 1.0) },
            CubeVert { v_pos: vec3(-1.0, -1.0,  1.0), v_color: vec4(0.0, 0.5, 1.0, 1.0) },
            CubeVert { v_pos: vec3(1.0, -1.0,  1.0), v_color: vec4(0.0, 0.5, 1.0, 1.0) },
            CubeVert { v_pos: vec3(1.0, -1.0, -1.0), v_color: vec4(0.0, 0.5, 1.0, 1.0) },

            CubeVert { v_pos: vec3(-1.0,  1.0, -1.0), v_color: vec4(1.0, 0.0, 0.5, 1.0) },
            CubeVert { v_pos: vec3(-1.0,  1.0,  1.0), v_color: vec4(1.0, 0.0, 0.5, 1.0) },
            CubeVert { v_pos: vec3(1.0,  1.0,  1.0),  v_color: vec4(1.0, 0.0, 0.5, 1.0) },
            CubeVert { v_pos: vec3(1.0,  1.0, -1.0),  v_color: vec4(1.0, 0.0, 0.5, 1.0) },
        ]);

        #[rustfmt::skip]
        let indicies_cube = ctx.new_index_buffer(BufferUsage::Immutable, &[
            0, 1, 2,  0, 2, 3,
            6, 5, 4,  7, 6, 4,
            8, 9, 10,  8, 10, 11,
            14, 13, 12,  15, 14, 12,
            16, 17, 18,  16, 18, 19,
            22, 21, 20,  23, 22, 20
        ]);

        let offscreen_pipeline = ctx.new_pipeline();

        App {
            vertices_cube,
            indicies_cube,
            offscreen_pipeline,
            offscreen_pass,
            rx: 0.,
            ry: 0.,
            ctx,
        }
    }
}

impl App {
    fn resize_event(&mut self, (width, height): (u32, u32)) {
        let color_img = self.ctx.new_empty_texture(
            width,
            height,
            Texture2DParams { internal_format: Texture2DFormat::RGBA8, ..Default::default() },
        );
        let depth_img = self.ctx.new_empty_texture(
            width,
            height,
            Texture2DParams { internal_format: Texture2DFormat::DepthU16, ..Default::default() },
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
        self.ctx
            .default_pass(Clear::depth_color(Color::WHITE), |_, _| {
                self.ctx.draw(DrawCall {
                    pipeline: &self.offscreen_pipeline,
                    base_element: 0,
                    num_elements: 36,
                    vertex_buffer: &self.vertices_cube,
                    index_buffer: &self.indicies_cube,
                    images: &NoImages,
                    uniforms: &OffscreenUniforms { mvp: view_proj * model },
                });
            });
    }
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
pub struct CubeVert {
    pub v_pos: Vec3,
    pub v_color: Vec4,
}

impl Vertex for CubeVert {
    const LAYOUT: &'static [VertexField] =
        &[attribute_of!(CubeVert, v_pos), attribute_of!(CubeVert, v_color)];
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy)]
pub struct QuadVert {
    pub v_pos: Vec2,
    pub v_uv: Vec2,
}

impl Vertex for QuadVert {
    const LAYOUT: &'static [VertexField] =
        &[attribute_of!(QuadVert, v_pos), attribute_of!(QuadVert, v_uv)];
}

pub struct OffscreenMeta;

impl PipelineMeta for OffscreenMeta {
    const VERTEX_SHADER: &str = include_str!("shaders/mvp_color.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/basic_color.frag");

    const IMAGES_NAMES: () = ();
    type Images = NoImages;
    type Vertex = CubeVert;
    type Uniforms = OffscreenUniforms;
    const PARAMS: PipelineParams =
        PipelineParams { depth_test: Some(Comparison::LessOrEqual), ..default_pipeline_params() };
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct OffscreenUniforms {
    pub mvp: glam::Mat4,
}

impl UniformBlock for OffscreenUniforms {
    const FIELDS: &'static [UniformField] = &[uniform_of!(OffscreenUniforms, mvp)];
}
