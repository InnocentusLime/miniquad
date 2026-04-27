//! A rendering example. You can spawn entities by
//! clicking. They should bounce around the screen
//! and visually interact with each other.
//! Should look like this:
//! https://youtu.be/W52jTDKOzIk

use mimiq::graphics::*;
use mimiq::*;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, vec2};
use std::rc::Rc;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf::default(), ());
}

struct App {
    mouse_pos: Vec2,
    pipeline: Pipeline<Meta>,
    vertices: VertexBuffer<BlobVertex>,
    indicies: IndexBuffer,
    uniforms: Uniforms,
    blobs_velocities: [(f32, f32); 32],
    ctx: Rc<GlContext>,
    total_time: Duration,
}

impl EventHandler<()> for App {
    fn update(&mut self, dt: Duration) {
        let delta = dt.as_secs_f32();
        self.total_time += dt;

        for i in 1..self.uniforms.blobs_count as usize {
            self.uniforms.blobs_positions[i].x += self.blobs_velocities[i].0 * delta * 0.1;
            self.uniforms.blobs_positions[i].y += self.blobs_velocities[i].1 * delta * 0.1;

            if self.uniforms.blobs_positions[i].x < 0. || self.uniforms.blobs_positions[i].x > 1. {
                self.blobs_velocities[i].0 *= -1.;
            }
            if self.uniforms.blobs_positions[i].y < 0. || self.uniforms.blobs_positions[i].y > 1. {
                self.blobs_velocities[i].1 *= -1.;
            }
        }
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.on_click(),
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_motion_event(vec2(position.x as f32, position.y as f32));
            }
            _ => (),
        }
    }

    fn init(ctx: Rc<GlContext>, _fs: Rc<dyn FsServer>, _init: ()) -> App {
        #[rustfmt::skip]
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &[
            BlobVertex { v_pos : Vec2 { x: -1.0, y: -1.0 }, v_uv: Vec2 { x: 0., y: 0. } },
            BlobVertex { v_pos : Vec2 { x:  1.0, y: -1.0 }, v_uv: Vec2 { x: 1., y: 0. } },
            BlobVertex { v_pos : Vec2 { x:  1.0, y:  1.0 }, v_uv: Vec2 { x: 1., y: 1. } },
            BlobVertex { v_pos : Vec2 { x: -1.0, y:  1.0 }, v_uv: Vec2 { x: 0., y: 1. } },
        ]);

        #[rustfmt::skip]
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &[
            0, 1, 2,
            0, 2, 3,
        ]);

        let pipeline = ctx.new_pipeline();

        let uniforms = Uniforms { time: 0., blobs_count: 1, blobs_positions: [vec2(0., 0.); 32] };

        App {
            pipeline,
            vertices,
            indicies,
            uniforms,
            mouse_pos: Vec2::ZERO,
            blobs_velocities: [(0., 0.); 32],
            ctx,
            total_time: Duration::ZERO,
        }
    }
}

impl App {
    fn mouse_motion_event(&mut self, pos: Vec2) {
        self.mouse_pos = pos;
        let Vec2 { x, y } = self.mouse_pos;
        let (w, h) = self.ctx.screen_size();
        let (w, h) = (w as f32, h as f32);
        let (x, y) = (x / w, 1. - y / h);
        self.uniforms.blobs_positions[0] = vec2(x, y);
    }

    fn on_click(&mut self) {
        if self.uniforms.blobs_count >= 32 {
            return;
        }

        let Vec2 { x, y } = self.mouse_pos;
        let (w, h) = self.ctx.screen_size();
        let (w, h) = (w as f32, h as f32);
        let (x, y) = (x / w, 1. - y / h);
        let (dx, dy) = (quad_rand::gen_range(-1., 1.), quad_rand::gen_range(-1., 1.));

        self.uniforms.blobs_positions[self.uniforms.blobs_count as usize] = vec2(x, y);
        self.blobs_velocities[self.uniforms.blobs_count as usize] = (dx, dy);
        self.uniforms.blobs_count += 1;
    }

    fn draw(&mut self) {
        self.uniforms.time = self.total_time.as_secs_f32();
        self.ctx
            .default_pass(Clear::depth_color(Color::BLACK), |_, _| {
                self.ctx.draw(DrawCall {
                    pipeline: &self.pipeline,
                    base_element: 0,
                    num_elements: 6,
                    vertex_buffer: &self.vertices,
                    index_buffer: &self.indicies,
                    images: &NoImages,
                    uniforms: &self.uniforms,
                });
            });
    }
}

#[repr(C)]
#[derive(Debug, Default, Zeroable, Pod, Clone, Copy, Vertex)]
pub struct BlobVertex {
    pub v_pos: Vec2,
    pub v_uv: Vec2,
}

// based on: https://www.shadertoy.com/view/XsS3DV
pub struct Meta;

impl PipelineMeta for Meta {
    const VERTEX_SHADER: &str = include_str!("shaders/basic_texture.vert");
    const FRAGMENT_SHADER: &str = include_str!("shaders/blobs.frag");

    const IMAGES_NAMES: () = ();
    type Images = NoImages;
    type Vertex = BlobVertex;
    type Uniforms = Uniforms;
    const PARAMS: PipelineParams = default_pipeline_params();
}

#[repr(C)]
#[derive(Debug, Zeroable, Pod, Clone, Copy, UniformBlock)]
pub struct Uniforms {
    pub time: f32,
    pub blobs_count: i32,
    pub blobs_positions: [Vec2; 32],
}
