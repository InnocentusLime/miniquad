use winit::{event::WindowEvent, window::Window};

/// A trait defining event callbacks.
pub trait EventHandler {
    /// On most platforms update() and draw() are called each frame, sequentially,
    /// draw right after update.
    /// But on Android (and maybe some other platforms in the future) update might
    /// be called without draw.
    /// When the app is in background, Android destroys the rendering surface,
    /// while app is still alive and can do some usefull calculations.
    /// Note that in this case drawing from update may lead to crashes.
    fn update(&mut self);

    fn window_event(&mut self, event: WindowEvent, window: &Window);
}
