//! A post processing example. Draws a rotating cube with
//! differently colored sides. Should look like this:
//! https://youtu.be/hdWWe-TkkfM

use mimiq::graphics::*;
use mimiq::*;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    vertices_cube: VertexBuffer<CubeVert>,
    indicies_cube: IndexBuffer,
    vertices_quad: VertexBuffer<QuadVert>,
    indicies_quad: IndexBuffer,
    post_processing_pipeline:
        Pipeline<QuadVert, PostProcessingUniforms, PostProcessingImages<'static>>,
    offscreen_pipeline: Pipeline<CubeVert, OffscreenUniforms>,
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
            WindowEvent::Resized(sz) => self.resize_event(sz.into()),
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: Rc<dyn FsServer>, _init: ()) -> App {
        let (width, height) = ctx.screen_size();
        let color_img = ctx
            .new_empty_texture(
                Texture2DFormat::RGBA8,
                width,
                height,
                TextureWrap::Clamp,
                FilterMode::Linear,
                FilterMode::Linear,
            )
            .unwrap();
        let depth_img = ctx
            .new_empty_texture(
                Texture2DFormat::DepthU16,
                width,
                height,
                TextureWrap::Clamp,
                FilterMode::Linear,
                FilterMode::Linear,
            )
            .unwrap();

        let offscreen_pass = ctx
            .new_render_pass(vec![color_img], Some(depth_img))
            .unwrap();

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
        ]).unwrap();

        #[rustfmt::skip]
        let indicies_cube = ctx.new_index_buffer(BufferUsage::Immutable, &[
            0, 1, 2,  0, 2, 3,
            6, 5, 4,  7, 6, 4,
            8, 9, 10,  8, 10, 11,
            14, 13, 12,  15, 14, 12,
            16, 17, 18,  16, 18, 19,
            22, 21, 20,  23, 22, 20
        ]).unwrap();

        #[rustfmt::skip]
        let vertices_quad = ctx.new_vertex_buffer(BufferUsage::Immutable, &[
            QuadVert { v_pos: vec2(-1.0, -1.0), v_uv: vec2(0.0, 0.0) },
            QuadVert { v_pos: vec2(1.0, -1.0), v_uv: vec2(1.0, 0.0) },
            QuadVert { v_pos: vec2(1.0,  1.0), v_uv: vec2(1.0, 1.0) },
            QuadVert { v_pos: vec2(-1.0,  1.0), v_uv: vec2(0.0, 1.0) },
        ]).unwrap();

        #[rustfmt::skip]
        let indicies_quad = ctx.new_index_buffer(BufferUsage::Immutable, &[
            0, 1, 2,
            0, 2, 3,
        ]).unwrap();

        let post_processing_pipeline = ctx
            .new_pipeline(
                include_str!("shaders/basic_texture.vert"),
                include_str!("shaders/gaus_blur.frag"),
                default_pipeline_params(),
            )
            .unwrap();
        let offscreen_pipeline = ctx
            .new_pipeline(
                include_str!("shaders/mvp_color.vert"),
                include_str!("shaders/basic_color.frag"),
                PipelineParams {
                    depth_test: Some(Comparison::LessOrEqual),
                    ..default_pipeline_params()
                },
            )
            .unwrap();

        App {
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

impl App {
    fn resize_event(&mut self, (width, height): (u32, u32)) {
        let color_img = self
            .ctx
            .new_empty_texture(
                Texture2DFormat::RGBA8,
                width,
                height,
                TextureWrap::Clamp,
                FilterMode::Linear,
                FilterMode::Linear,
            )
            .unwrap();
        let depth_img = self
            .ctx
            .new_empty_texture(
                Texture2DFormat::DepthU16,
                width,
                height,
                TextureWrap::Clamp,
                FilterMode::Linear,
                FilterMode::Linear,
            )
            .unwrap();

        self.offscreen_pass =
            RenderPass::new(self.ctx.clone(), vec![color_img], Some(depth_img)).unwrap();
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
            .pass(Clear::depth_color(Color::WHITE), |_, _| {
                self.offscreen_pipeline.draw(
                    0,
                    36,
                    &self.vertices_cube,
                    &self.indicies_cube,
                    &NoImages,
                    &OffscreenUniforms { mvp: view_proj * model },
                )
            })
            .unwrap();

        // and the post-processing-pass, rendering a rotating, textured cube, using the
        // previously rendered offscreen render-target as texture
        self.ctx
            .default_pass(Clear::depth_color(Color::WHITE), |_, _| {
                self.post_processing_pipeline.draw(
                    0,
                    6,
                    &self.vertices_quad,
                    &self.indicies_quad,
                    &PostProcessingImages { tex: &self.offscreen_pass.color_attachments()[0] },
                    &PostProcessingUniforms { resolution: vec2(width, height) },
                )
            })
            .unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy, Vertex)]
pub struct CubeVert {
    pub v_pos: Vec3,
    pub v_color: Vec4,
}

#[repr(C)]
#[derive(Debug, Default, Pod, Zeroable, Clone, Copy, Vertex)]
pub struct QuadVert {
    pub v_pos: Vec2,
    pub v_uv: Vec2,
}

#[derive(Debug, Clone, Copy, ImagesUniformBlock)]
pub struct PostProcessingImages<'a> {
    pub tex: &'a Texture2D,
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy, UniformBlock)]
pub struct PostProcessingUniforms {
    pub resolution: glam::Vec2,
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy, UniformBlock)]
pub struct OffscreenUniforms {
    pub mvp: glam::Mat4,
}
