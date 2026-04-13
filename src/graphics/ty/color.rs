use bytemuck::{Pod, Zeroable};
use glam::{Vec4, vec4};

pub const fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(r, g, b, a)
}

pub const fn color_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

pub const fn color_hex(hex: u32) -> Color {
    Color::from_hex(hex)
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLANK: Color = color(0.00, 0.00, 0.00, 0.00);

    pub const CYAN: Color = color(0.1, 0.89, 0.9, 1.00);
    pub const LIGHTGRAY: Color = color(0.78, 0.78, 0.78, 1.00);
    pub const GRAY: Color = color(0.51, 0.51, 0.51, 1.00);
    pub const DARKGRAY: Color = color(0.31, 0.31, 0.31, 1.00);
    pub const YELLOW: Color = color(0.99, 0.98, 0.00, 1.00);
    pub const GOLD: Color = color(1.00, 0.80, 0.00, 1.00);
    pub const ORANGE: Color = color(1.00, 0.63, 0.00, 1.00);
    pub const PINK: Color = color(1.00, 0.43, 0.76, 1.00);
    pub const RED: Color = color(0.90, 0.16, 0.22, 1.00);
    pub const MAROON: Color = color(0.75, 0.13, 0.22, 1.00);
    pub const GREEN: Color = color(0.00, 0.89, 0.19, 1.00);
    pub const LIME: Color = color(0.00, 0.62, 0.18, 1.00);
    pub const DARKGREEN: Color = color(0.00, 0.46, 0.17, 1.00);
    pub const SKYBLUE: Color = color(0.40, 0.75, 1.00, 1.00);
    pub const BLUE: Color = color(0.00, 0.47, 0.95, 1.00);
    pub const DARKBLUE: Color = color(0.00, 0.32, 0.67, 1.00);
    pub const PURPLE: Color = color(0.78, 0.48, 1.00, 1.00);
    pub const VIOLET: Color = color(0.53, 0.24, 0.75, 1.00);
    pub const DARKPURPLE: Color = color(0.44, 0.12, 0.49, 1.00);
    pub const BEIGE: Color = color(0.83, 0.69, 0.51, 1.00);
    pub const BROWN: Color = color(0.50, 0.42, 0.31, 1.00);
    pub const DARKBROWN: Color = color(0.30, 0.25, 0.18, 1.00);
    pub const WHITE: Color = color(1.00, 1.00, 1.00, 1.00);
    pub const BLACK: Color = color(0.00, 0.00, 0.00, 1.00);
    pub const MAGENTA: Color = color(1.00, 0.00, 1.00, 1.00);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub const fn to_vec4(self) -> Vec4 {
        vec4(self.r, self.g, self.b, self.a)
    }

    pub const fn from_vec4(v: Vec4) -> Self {
        color(v.x, v.y, v.z, v.w)
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        color(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    pub const fn from_hex(hex: u32) -> Color {
        let [_, r, g, b] = hex.to_be_bytes();
        Self::from_rgba(r, g, b, 255)
    }
}
