use std::error::Error as StdError;
use std::fmt;

use glow::HasContext;

#[macro_export]
macro_rules! check_gl {
    ($ctx:expr, $variant:path) => {
        $crate::graphics::check_gl($ctx, file!(), line!()).map_err($variant)?;
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FramebufferAlloc,
    FramebufferInit(GlCheckError),
    BufferAlloc,
    BufferInit(GlCheckError),
    TextureAlloc,
    TextureInit(GlCheckError),
    UnsupportedTextureFormat { format: image::ColorType },
    Drawcall(GlCheckError),
    ShaderCompilation { shader_type: &'static str, msg: String },
    ProgramLink { msg: String },
    UniformNotFound { name: String },
    VertexAttributeMismatch { name: String, expected: u32, found: u32 },
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn StdError> {
        self.source()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FramebufferAlloc => write!(f, "Failed to allocate a framebuffer"),
            Error::FramebufferInit(e) => write!(f, "Failed to init a framebuffer: {e}"),
            Error::BufferAlloc => write!(f, "Failed to allocate a buffer"),
            Error::BufferInit(e) => write!(f, "Failed to init a buffer: {e}"),
            Error::TextureAlloc => write!(f, "Failed to allocate a texture"),
            Error::TextureInit(e) => write!(f, "Failed to init a texture: {e}"),
            Error::UnsupportedTextureFormat { format } => {
                write!(f, "Usupported texture format: {format:?}")
            }
            Error::Drawcall(e) => write!(f, "Drawcall failed: {e}"),
            Error::ShaderCompilation { shader_type, msg } => {
                write!(f, "Failed to compiler shader type {shader_type:?}:\n{msg}")
            }
            Error::ProgramLink { msg } => write!(f, "Failed to link the program:\n{msg}"),
            Error::UniformNotFound { name } => write!(f, "Uniform not found: {name}"),
            Error::VertexAttributeMismatch { name, expected, found } => write!(
                f,
                "Vertex attribute {name:?} index mismatch: expected {expected}, found {found}"
            ),
        }
    }
}

#[derive(Debug)]
pub struct GlCheckError {
    pub file: &'static str,
    pub line: u32,
    pub gl_err: u32,
}

impl fmt::Display for GlCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GL check failed at {}:{}: {:x}",
            self.file, self.line, self.gl_err
        )
    }
}

pub fn check_gl(
    ctx: &glow::Context,
    file: &'static str,
    line: u32,
) -> std::result::Result<(), GlCheckError> {
    let gl_err = unsafe { ctx.get_error() };
    if gl_err == glow::NO_ERROR { Ok(()) } else { Err(GlCheckError { file, line, gl_err }) }
}
