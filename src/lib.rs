#![doc = include_str!("../README.md")]
#![allow(
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::unused_unit,
    clippy::identity_op,
    clippy::missing_safety_doc
)]

pub mod conf;
mod event;
pub mod fs;
pub mod graphics;
pub mod native;

pub use event::*;

pub use graphics::*;

mod default_icon;

pub use bytemuck::offset_of;

pub mod date {
    pub fn now() -> f64 {
        use std::time::SystemTime;

        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|e| panic!("{}", e));
        time.as_secs_f64()
    }
}

use std::rc::Rc;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum CursorIcon {
    Default,
    Help,
    Pointer,
    Wait,
    Crosshair,
    Text,
    Move,
    NotAllowed,
    EWResize,
    NSResize,
    NESWResize,
    NWSEResize,
}

/// Start miniquad.
pub fn start<F>(conf: conf::Conf, f: F)
where
    F: 'static + FnOnce(Rc<GlContext>) -> Box<dyn EventHandler>,
{
    native::run(conf, f);
}
