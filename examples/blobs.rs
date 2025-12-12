use bytemuck::{Pod, Zeroable};
use glam::{Vec2, vec2};
use miniquad::*;
///! A rendering example. You can spawn entities by
///! clicking. They should bounce around the screen
///! and visually interact with each other.
///! Should look like this:
///! https://youtu.be/W52jTDKOzIk
use std::rc::Rc;
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    window::Window,
};

fn main() {
    miniquad::run::<Stage>(Conf::default());
}

struct Stage {
    mouse_pos: Vec2,
    pipeline: Pipeline<shader::Uniforms>,
    vertices: VertexBuffer<Vertex>,
    indicies: IndexBuffer,
    uniforms: shader::Uniforms,
    blobs_velocities: [(f32, f32); 32],
    ctx: Rc<GlContext>,
    start: Instant,
    last_frame: Instant,
}

impl EventHandler for Stage {
    fn update(&mut self) {
        let new_frame = Instant::now();
        let delta = new_frame.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = new_frame;

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

    fn init(ctx: Rc<GlContext>, _fs: FsServerHandle) -> Stage {
        #[rustfmt::skip]
        let vertices = [
            Vertex { pos : Vec2 { x: -1.0, y: -1.0 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  1.0, y: -1.0 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  1.0, y:  1.0 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -1.0, y:  1.0 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertices = ctx.new_vertex_buffer(BufferUsage::Immutable, &vertices);

        let indicies = [0, 1, 2, 0, 2, 3];
        let indicies = ctx.new_index_buffer(BufferUsage::Immutable, &indicies);

        let pipeline = ctx
            .new_pipeline(
                shader::VERTEX,
                shader::FRAGMENT,
                PipelineParams::default(),
                [
                    VertexAttribute::new("in_pos", VertexFormat::F32x2),
                    VertexAttribute::new("in_uv", VertexFormat::F32x2),
                ],
                [
                    UniformDesc::new_scalar("time", UniformType::F32),
                    UniformDesc::new_scalar("blobs_count", UniformType::I32),
                    UniformDesc::new_array("blobs_positions", UniformType::F32x2, 32),
                ],
                [],
            )
            .unwrap();

        let uniforms = shader::Uniforms {
            time: 0.,
            blobs_count: 1,
            blobs_positions: [vec2(0., 0.); 32],
        };

        let time = Instant::now();
        Stage {
            pipeline,
            vertices,
            indicies,
            uniforms,
            mouse_pos: Vec2::ZERO,
            blobs_velocities: [(0., 0.); 32],
            ctx,
            last_frame: time,
            start: time,
        }
    }
}

impl Stage {
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
        self.uniforms.time = self.last_frame.duration_since(self.start).as_secs_f32();
        self.ctx
            .perform_default_render_pass(Clear::depth_color(BLACK), |_, _| {
                self.ctx.submit_drawcall(DrawCall {
                    pipeline: &self.pipeline,
                    base_element: 0,
                    num_elements: 6,
                    vertex_buffers: &bind_vertex_buffers![
                        (&self.vertices) as <Vertex>::pos,
                        (&self.vertices) as <Vertex>::uv,
                    ],
                    index_buffer: self.indicies.bind(),
                    textures: &[],
                    uniforms: &self.uniforms,
                });
            });
    }
}

#[repr(C)]
#[derive(Default, Zeroable, Pod, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

// based on: https://www.shadertoy.com/view/XsS3DV
mod shader {
    use bytemuck::{Pod, Zeroable};
    use glam::Vec2;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    varying highp vec2 uv;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        uv = in_uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    precision highp float;

    varying vec2 uv;

    uniform float time;
    uniform int blobs_count;
    uniform vec2 blobs_positions[32];

    float k = 20.0;
    float field = 0.0;
    vec2 coord;
        
    void circle ( float r , vec3 col , vec2 offset) {
        vec2 pos = coord.xy;
        vec2 c = offset;
        float d = distance ( pos , c );
        field += ( k * r ) / ( d*d );
    }
        
    vec3 band ( float shade, float low, float high, vec3 col1, vec3 col2 ) {
        if ( (shade >= low) && (shade <= high) ) {
            float delta = (shade - low) / (high - low);
            vec3 colDiff = col2 - col1;
            return col1 + (delta * colDiff);
        }
        else
            return vec3(0.0,0.0,0.0);
    }
    
    vec3 gradient ( float shade ) {
        vec3 colour = vec3( (sin(time/2.0)*0.25)+0.25,0.0,(cos(time/2.0)*0.25)+0.25);
        
        vec3 col1 = vec3(0.01, 0.0, 1.0-0.01);
        vec3 col2 = vec3(1.0-0.01, 0.0, 0.01);
        vec3 col3 = vec3(0.02, 1.0-0.02, 0.02);
        vec3 col4 = vec3((0.01+0.02)/2.0, (0.01+0.02)/2.0, 1.0 - (0.01+0.02)/2.0);
        vec3 col5 = vec3(0.02, 0.02, 0.02);
        
        colour += band ( shade, 0.0, 0.3, colour, col1 );
        colour += band ( shade, 0.3, 0.6, col1, col2 );
        colour += band ( shade, 0.6, 0.8, col2, col3 );
        colour += band ( shade, 0.8, 0.9, col3, col4 );
        colour += band ( shade, 0.9, 1.0, col4, col5 );
        
        return colour;
    }
    
    void main() {
        coord = uv;
        
        for (int i = 0; i < 32; i++) {
            if (i >= blobs_count) { break; } // workaround for webgl error: Loop index cannot be compared with non-constant expression
            circle(.03 , vec3(0.7 ,0.2, 0.8), blobs_positions[i]);
        }
        
        float shade = min ( 1.0, max ( field/256.0, 0.0 ) );
        
        gl_FragColor = vec4( gradient(shade), 1.0 );
    }"#;

    #[repr(C)]
    #[derive(Zeroable, Pod, Clone, Copy)]
    pub struct Uniforms {
        pub time: f32,
        pub blobs_count: i32,
        pub blobs_positions: [Vec2; 32],
    }
}
