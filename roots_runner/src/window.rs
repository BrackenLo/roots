//====================================================================

use std::sync::Arc;

use roots_common::Size;
use winit::{event_loop::ActiveEventLoop, window::WindowAttributes};

//====================================================================

pub struct Window(Arc<winit::window::Window>);
impl Window {
    pub fn new(event_loop: &ActiveEventLoop, window_attributes: Option<WindowAttributes>) -> Self {
        log::info!("Creating new window");

        let window = event_loop
            .create_window(window_attributes.unwrap_or_default())
            .unwrap();

        Self(Arc::new(window))
    }

    #[inline]
    pub fn size(&self) -> Size<u32> {
        let window_size = self.0.inner_size();

        Size {
            width: window_size.width,
            height: window_size.height,
        }
    }

    #[inline]
    pub fn confine_cursor(&self, confined: bool) {
        log::trace!("Confining window cursor: {}", confined);

        self.0
            .set_cursor_grab(match confined {
                true => winit::window::CursorGrabMode::Confined,
                false => winit::window::CursorGrabMode::None,
            })
            .unwrap();
    }

    #[inline]
    pub fn hide_cursor(&self, hidden: bool) {
        log::trace!("Hiding window cursor: {}", hidden);
        self.0.set_cursor_visible(!hidden);
    }

    #[inline]
    pub fn inner(&self) -> &winit::window::Window {
        &self.0
    }

    #[inline]
    pub fn arc(&self) -> Arc<winit::window::Window> {
        self.0.clone()
    }
}

//====================================================================
