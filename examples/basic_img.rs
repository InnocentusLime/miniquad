use bytemuck::{Pod, Zeroable};
use glam::{Vec2, vec2};
use miniquad::{fs::FsServerHandle, *};
///! A simple rendering example. This example loads a texture from memory
///! and draws a few quads with it. The example should look as follows:
///! https://youtu.be/kksaeWrAT7E
use std::rc::Rc;
use winit::{event::WindowEvent, window::Window};

fn main() {
    miniquad::start(Conf::default(), Stage::new);
}

struct Stage {
    ctx: Rc<GlContext>,

    pipeline: Pipeline,
    vertices: Buffer<Vertex>,
    indicies: IndexBuffer,
    texture: Texture,
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }
}

impl Stage {
    pub fn new(ctx: Rc<GlContext>, fs: FsServerHandle) -> Stage {
        let handle = fs.submit_task("./examples/assets/ferris.png", |img_bytes| {
            let img = image::load_from_memory(&img_bytes)?.flipv();
            let parsed_bytes = img.to_rgba8().into_vec();
            Ok((img.width(), img.height(), parsed_bytes))
        });
        let (width, height, img) = loop {
            let Some(x) = handle.is_done() else { continue; };
            break x.unwrap();
        };

        #[rustfmt::skip]
        let vertices = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertices = ctx.new_buffer(BufferUsage::Immutable, &vertices);

        let indicies = [0, 1, 2, 0, 2, 3];
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &indicies);

        let texture = ctx.new_texture(
            TextureSource::Bytes(&img),
            TextureParams {
                format: TextureFormat::RGBA8,
                width,
                height,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                allocate_mipmaps: false,
            },
        );

        let pipeline = ctx
            .new_pipeline(
                shader::VERTEX,
                shader::FRAGMENT,
                PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
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
            vertices,
            indicies,
            texture,
            ctx,
        }
    }

    fn draw(&mut self) {
        let t = date::now();

        self.ctx.perform_default_render_pass(
            PassAction::clear_depth_color(0.0, 0.0, 0.0, 1.0),
            || {
                for i in 0..10 {
                    let t = t + i as f64 * 0.3;
                    let uniforms = shader::Uniforms {
                        offset: vec2(t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5),
                    };
                    DrawCall {
                        ctx: &self.ctx,
                        pipeline: &self.pipeline,
                        base_element: 0,
                        num_elements: 6,
                        vertex_buffers: &bind_buffers![
                            (&self.vertices) as <Vertex>::pos,
                            (&self.vertices) as <Vertex>::uv,
                        ],
                        index_buffer: self.indicies.bind(),
                        textures: &[self.texture.bind()],
                        uniform_data: bytemuck::bytes_of(&uniforms),
                    }
                    .execute();
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
