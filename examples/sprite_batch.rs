//! A simple rendering example, similar to quad, but instead
//! loads an image from file, using file loading capabilities.

use core::f32;
use glam::{Affine2, Mat4, uvec2, vec2};
use mimiq::util::{BasicSpritePipelineUniforms, SpriteBatcher};
use mimiq::*;
use std::path::Path;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::window::Window;

fn main() {
    mimiq::run::<(), App>(Conf { fs_root: "examples/".into(), ..Conf::default() }, ());
}

struct App {
    total_time: Duration,
    ctx: Rc<GlContext>,

    pipeline: Pipeline<util::BasicSpritePipelineMeta>,
    batcher: util::SpriteBatcher,
    texture: Option<Texture2D>,
}

impl EventHandler<()> for App {
    fn update(&mut self, dt: Duration) {
        self.total_time += dt;
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }

    fn file_ready(&mut self, event: FileReady) {
        let Ok(bytes) = event.bytes_result else {
            return;
        };
        let img = image::load_from_memory(&bytes).expect("Image load failed");
        self.texture = Some(self.ctx.new_texture(
            img,
            Texture2DParams {
                internal_format: Texture2DFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Nearest,
                mag_filter: FilterMode::Nearest,
            },
        ));
    }

    fn init(ctx: Rc<GlContext>, fs_server: Rc<dyn FsServer>, _init: ()) -> App {
        fs_server.load_file(Path::new("assets/GB-Tileset.png"));

        let pipeline = ctx.new_pipeline();
        let batcher = util::SpriteBatcher::new_from_size(&ctx, 2000);

        App { total_time: Duration::ZERO, batcher, pipeline, texture: None, ctx }
    }
}

impl App {
    fn draw(&mut self) {
        let t = self.total_time.as_secs_f32();

        let Some(texture) = self.texture.as_ref() else {
            return;
        };

        let char_x = 16.0 + (t.sin() + 1.0) * 0.5 * 48.0;
        let saw_x = t * 3.0;
        let saw = 4.0 * (saw_x.fract() - 0.5).abs() - 1.0;
        let char_tf = Affine2::from_angle_translation(
            saw * f32::consts::FRAC_PI_6 * t.cos(),
            vec2(char_x, 24.0),
        );

        self.ctx
            .default_pass(Clear::depth_color(Color::BLACK), |width, height| {
                let view_projection = Mat4::orthographic_rh_gl(
                    0.0,
                    width as f32 / 8.0,
                    height as f32 / 8.0,
                    0.0,
                    0.0,
                    100.0,
                );

                self.batcher.clear();
                put_tilemap(&mut self.batcher);
                self.batcher.add_sprite(util::Sprite {
                    tex_rect_pos: uvec2(48, 192),
                    tex_rect_size: uvec2(16, 16),
                    color: Color::WHITE,
                    transform: Affine2::from_translation(vec2(16.0, 16.0)),
                });
                self.batcher.add_sprite(util::Sprite {
                    tex_rect_pos: uvec2(64, 208),
                    tex_rect_size: uvec2(17, 16),
                    color: Color::WHITE,
                    transform: Affine2::from_translation(vec2(64.0, 16.0)),
                });

                self.batcher.add_sprite(util::Sprite {
                    tex_rect_pos: uvec2(64, 48),
                    tex_rect_size: uvec2(16, 16),
                    color: Color::WHITE,
                    transform: char_tf,
                });
                let num_elements = self.batcher.flush();
                self.ctx.draw(DrawCall {
                    pipeline: &self.pipeline,
                    base_element: 0,
                    num_elements,
                    vertex_buffer: &self.batcher.0.vertices,
                    index_buffer: &self.batcher.0.indicies,
                    images: texture,
                    uniforms: &BasicSpritePipelineUniforms {
                        view_projection,
                        width_height: texture.size().as_vec2(),
                    },
                });
            });
    }
}

fn put_tilemap(batcher: &mut SpriteBatcher) {
    let tiles = [
        util::Sprite {
            tex_rect_pos: uvec2(32, 192),
            tex_rect_size: uvec2(16, 16),
            color: Color::WHITE,
            transform: Affine2::IDENTITY,
        },
        util::Sprite {
            tex_rect_pos: uvec2(32, 176),
            tex_rect_size: uvec2(16, 16),
            color: Color::WHITE,
            transform: Affine2::IDENTITY,
        },
        util::Sprite {
            tex_rect_pos: uvec2(16, 176),
            tex_rect_size: uvec2(16, 16),
            color: Color::WHITE,
            transform: Affine2::IDENTITY,
        },
    ];

    let side = 16;
    for x in 0..side {
        for y in 0..side {
            batcher.add_sprite(util::Sprite {
                transform: Affine2::from_translation(vec2(
                    x as f32 * 16.0 + 8.0,
                    y as f32 * 16.0 + 8.0,
                )),
                ..tiles[(x ^ 6 * y) % 3]
            });
        }
    }
}
