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

#[derive(Debug, Clone, Copy, Pod, Zeroable, Default, PartialEq)]
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

#[cfg(feature = "serde")]
impl serde_core::Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        use serde_core::ser::SerializeTupleStruct;

        let mut state = serializer.serialize_tuple_struct("Color", 4)?;
        state.serialize_field(&self.r)?;
        state.serialize_field(&self.g)?;
        state.serialize_field(&self.b)?;
        state.serialize_field(&self.a)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde_core::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde_core::Deserializer<'de>,
    {
        static COLOR_NAMES: &'static [&str] = &[
            "BLANK",
            "CYAN",
            "LIGHTGRAY",
            "GRAY",
            "DARKGRAY",
            "YELLOW",
            "GOLD",
            "ORANGE",
            "PINK",
            "RED",
            "MAROON",
            "GREEN",
            "LIME",
            "DARKGREEN",
            "SKYBLUE",
            "BLUE",
            "DARKBLUE",
            "PURPLE",
            "VIOLET",
            "DARKPURPLE",
            "BEIGE",
            "BROWN",
            "DARKBROWN",
            "WHITE",
            "BLACK",
            "MAGENTA",
        ];

        static COLOR_VALUES: &'static [Color] = &[
            Color::BLANK,
            Color::CYAN,
            Color::LIGHTGRAY,
            Color::GRAY,
            Color::DARKGRAY,
            Color::YELLOW,
            Color::GOLD,
            Color::ORANGE,
            Color::PINK,
            Color::RED,
            Color::MAROON,
            Color::GREEN,
            Color::LIME,
            Color::DARKGREEN,
            Color::SKYBLUE,
            Color::BLUE,
            Color::DARKBLUE,
            Color::PURPLE,
            Color::VIOLET,
            Color::DARKPURPLE,
            Color::BEIGE,
            Color::BROWN,
            Color::DARKBROWN,
            Color::WHITE,
            Color::BLACK,
            Color::MAGENTA,
        ];

        struct ColorVisitor;

        impl<'de> serde_core::de::Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(&concat!("a sequence of 4 f32 values or a string"))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde_core::de::SeqAccess<'de>,
            {
                let r = seq
                    .next_element()?
                    .ok_or_else(|| serde_core::de::Error::invalid_length(0, &self))?;
                let g = seq
                    .next_element()?
                    .ok_or_else(|| serde_core::de::Error::invalid_length(1, &self))?;
                let b = seq
                    .next_element()?
                    .ok_or_else(|| serde_core::de::Error::invalid_length(2, &self))?;
                let a = seq
                    .next_element()?
                    .ok_or_else(|| serde_core::de::Error::invalid_length(3, &self))?;
                Ok(color(r, g, b, a))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde_core::de::Error,
            {
                match COLOR_NAMES
                    .iter()
                    .position(|x| (*x).eq_ignore_ascii_case(v))
                {
                    Some(id) => Ok(COLOR_VALUES[id]),
                    None => Err(E::unknown_variant(v, COLOR_NAMES)),
                }
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}

#[cfg(feature = "serde")]
mod serde_test {
    #[test]
    fn color_names() {
        use super::Color;

        assert_eq!(
            serde_json::from_str::<Color>("\"Red\"").unwrap(),
            Color::RED
        );
        assert_eq!(
            serde_json::from_str::<Color>("\"RED\"").unwrap(),
            Color::RED
        );
        assert_eq!(
            serde_json::from_str::<Color>("\"red\"").unwrap(),
            Color::RED
        );
        assert_eq!(
            serde_json::from_str::<Color>("\"YELLOW\"").unwrap(),
            Color::YELLOW
        );
    }

    #[test]
    fn color_ser_de_vector() {
        use super::Color;

        let x = Color { r: 1.0, g: 0.5, b: 0.4, a: 0.23 };
        let str = serde_json::to_string(&x).unwrap();
        assert_eq!(x, serde_json::from_str(&str).unwrap());
    }
}
