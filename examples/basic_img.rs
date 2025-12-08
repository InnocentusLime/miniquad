use bytemuck::{Pod, Zeroable};
use glam::{Vec2, vec2};
use miniquad::*;
///! A simple rendering example. This example loads a texture from memory
///! and draws a few quads with it. The example should look as follows:
///! https://youtu.be/kksaeWrAT7E
use std::rc::Rc;
use winit::{event::WindowEvent, window::Window};

fn main() {
    miniquad::run::<Stage>(Conf {
        fs_root: "examples/".into(),
        ..Conf::default()
    });
}

struct Stage {
    start: Instant,
    ctx: Rc<GlContext>,
    _fs_server: FsServerHandle,

    pipeline: Pipeline<shader::Uniforms>,
    vertices: VertexBuffer<Vertex>,
    indicies: IndexBuffer,
    texture: Option<Texture>,
}

impl EventHandler for Stage {
    fn update(&mut self) {}

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
            TextureParams {
                internal_format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
            },
        ));
    }

    fn init(ctx: Rc<GlContext>, fs_server: FsServerHandle) -> Stage {
        fs_server.submit_task("assets/ferris.png", 0);

        #[rustfmt::skip]
        let vertices = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &vertices);

        let indicies = [0, 1, 2, 0, 2, 3];
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &indicies);

        let pipeline = ctx
            .new_pipeline(
                shader::VERTEX,
                shader::FRAGMENT,
                PipelineParams {
                    blending: Blending::All(BlendFunc {
                        equation: BlendEquation::Add,
                        source: BlendFactor::Value(BlendValue::SrcAlpha),
                        dest: BlendFactor::OneMinusValue(BlendValue::SrcAlpha),
                    }),
                    ..Default::default()
                },
                [
                    VertexAttribute::new("in_pos", VertexFormat::F32x2),
                    VertexAttribute::new("in_uv", VertexFormat::F32x2),
                ],
                [UniformDesc::new_scalar("offset", UniformType::F32x2)],
                ["tex"],
            )
            .unwrap();

        Stage {
            pipeline,
            _fs_server: fs_server,
            vertices,
            indicies,
            texture: None,
            ctx,
            start: Instant::now(),
        }
    }
}

impl Stage {
    fn draw(&mut self) {
        let t = Instant::now().duration_since(self.start).as_secs_f32();

        let Some(texture) = self.texture.as_ref() else {
            return;
        };

        self.ctx.perform_default_render_pass(
            PassAction::clear_depth_color(0.0, 0.0, 0.0, 1.0),
            |_, _| {
                for i in 0..10 {
                    let t = t + i as f32 * 0.3;
                    self.ctx.submit_drawcall(DrawCall {
                        pipeline: &self.pipeline,
                        base_element: 0,
                        num_elements: 6,
                        vertex_buffers: &bind_vertex_buffers![
                            (&self.vertices) as <Vertex>::pos,
                            (&self.vertices) as <Vertex>::uv,
                        ],
                        index_buffer: self.indicies.bind(),
                        textures: &[texture.bind()],
                        uniforms: &shader::Uniforms {
                            offset: vec2(t.sin() * 0.5, (t * 3.).cos() * 0.5),
                        },
                    });
                }
            },
        );
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

mod shader {
    use bytemuck::{Pod, Zeroable};
    use glam::Vec2;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos + offset, 0, 1);
        texcoord = in_uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

    #[repr(C)]
    #[derive(Zeroable, Pod, Clone, Copy)]
    pub struct Uniforms {
        pub offset: Vec2,
    }
}
