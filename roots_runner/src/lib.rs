//====================================================================

use roots_common::Size;
use winit::{
    event::{DeviceEvent, DeviceId, MouseButton, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::WindowId,
};

pub use winit;
pub mod prelude {
    pub use winit::{
        event::{DeviceEvent, DeviceId, MouseButton, StartCause, WindowEvent},
        event_loop::ActiveEventLoop,
        keyboard::KeyCode,
        window::WindowId,
    };

    pub use crate::{runner::Runner, window::Window, RunnerState};
}

pub mod runner;
pub mod window;

//====================================================================

pub enum WindowInputEvent {
    KeyInput { key: KeyCode, pressed: bool },
    MouseInput { button: MouseButton, pressed: bool },
    CursorMoved { position: (f64, f64) },
    CursorEntered,
    CursorLeft,
    MouseWheel { delta: (f32, f32) },
    MouseMotion { delta: (f64, f64) },
}

pub trait RunnerState {
    fn new(event_loop: &ActiveEventLoop) -> Self;

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: &WindowEvent,
    ) {
        let _ = (event_loop, window_id, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: &DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause);

    fn input_event(&mut self, event: WindowInputEvent);

    fn resized(&mut self, new_size: Size<u32>);

    fn close_requested(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Close requested. Closing App.");
        event_loop.exit();
    }

    fn tick(&mut self, event_loop: &ActiveEventLoop);
}

//====================================================================

//====================================================================
