//====================================================================

use roots_common::input;
use roots_runner::{
    prelude::StartCause, window::Window, winit::event_loop::ControlFlow, WindowInputEvent,
};

use crate::{HecsApp, State, StateOuter};

//====================================================================

impl<A: HecsApp> roots_runner::RunnerState for StateOuter<A> {
    fn new(event_loop: &roots_runner::prelude::ActiveEventLoop) -> Self {
        let window = Window::new(event_loop, None);
        let mut state = State::new(window);

        let app = A::new(&mut state);

        Self { state, app }
    }

    fn new_events(
        &mut self,
        _event_loop: &roots_runner::prelude::ActiveEventLoop,
        cause: roots_runner::prelude::StartCause,
    ) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.state.window.inner().request_redraw();
        }
    }

    fn input_event(&mut self, event: roots_runner::WindowInputEvent) {
        match event {
            WindowInputEvent::KeyInput { key, pressed } => {
                input::process_inputs(&mut self.state.keys, key, pressed)
            }
            WindowInputEvent::MouseInput { button, pressed } => {
                input::process_inputs(&mut self.state.mouse_buttons, button, pressed)
            }
            WindowInputEvent::CursorMoved { position } => {
                input::process_mouse_position(&mut self.state.mouse_input, position)
            }
            WindowInputEvent::CursorEntered => {}
            WindowInputEvent::CursorLeft => {}
            WindowInputEvent::MouseWheel { delta } => {
                input::process_mouse_scroll(&mut self.state.mouse_input, delta)
            }
            WindowInputEvent::MouseMotion { delta } => {
                input::process_mouse_motion(&mut self.state.mouse_input, delta)
            }
        }
    }

    fn resized(&mut self, new_size: roots_common::Size<u32>) {
        log::trace!("Resizing window. New size = {}", new_size);
        self.app.resize(&mut self.state, new_size);
        self.state.renderer.resize(new_size);
    }

    fn tick(&mut self, event_loop: &roots_runner::prelude::ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::wait_duration(self.state.target_fps));

        roots_common::tick_time(&mut self.state.time);

        self.app.tick(&mut self.state);

        roots_common::input::reset_input(&mut self.state.keys);
        roots_common::input::reset_input(&mut self.state.mouse_buttons);
        roots_common::input::reset_mouse_input(&mut self.state.mouse_input);
    }
}

//====================================================================
