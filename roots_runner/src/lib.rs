//====================================================================

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    window::WindowId,
};

pub use winit;

//====================================================================

pub trait State {
    fn new(event_loop: &ActiveEventLoop) -> Self;

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    );

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    );

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause);
}

//====================================================================

pub struct Runner<S: State> {
    state: Option<S>,
}

impl<S: State> ApplicationHandler for Runner<S> {
    #[inline]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::trace!("App/Window Resumed - Creating State.");

        match self.state {
            Some(_) => log::warn!("State already exists"),
            None => self.state = Some(S::new(event_loop)),
        }
    }

    #[inline]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            state.window_event(event_loop, window_id, event);
        }
    }

    #[inline]
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let Some(state) = &mut self.state {
            state.new_events(event_loop, cause);
        }
    }

    #[inline]
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = &mut self.state {
            state.device_event(event_loop, device_id, event);
        }
    }
}

//====================================================================
