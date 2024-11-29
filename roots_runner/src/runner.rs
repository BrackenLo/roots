//====================================================================

use roots_common::Size;
use winit::application::ApplicationHandler;

use crate::{Runner, RunnerState, WindowInputEvent};

//====================================================================

impl<S: RunnerState> ApplicationHandler for Runner<S> {
    #[inline]
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::trace!("App/Window Resumed - Creating State.");

        match self.state {
            Some(_) => log::warn!("State already exists"),
            None => self.state = Some(S::new(event_loop)),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(runner_state) = &mut self.state {
            runner_state.window_event(event_loop, window_id, &event);

            match event {
                winit::event::WindowEvent::Resized(new_size) => {
                    runner_state.resized(Size::new(new_size.width, new_size.height))
                }

                //--------------------------------------------------
                //
                winit::event::WindowEvent::CloseRequested => {
                    runner_state.close_requested(event_loop)
                }
                winit::event::WindowEvent::Destroyed => {
                    log::warn!("Window was reported destroyed");
                    runner_state.close_requested(event_loop);
                }

                //--------------------------------------------------
                //
                winit::event::WindowEvent::KeyboardInput { event, .. } => {
                    if let winit::keyboard::PhysicalKey::Code(key) = event.physical_key {
                        runner_state.input_event(WindowInputEvent::KeyInput {
                            key,
                            pressed: event.state.is_pressed(),
                        });
                    }
                }

                winit::event::WindowEvent::CursorMoved { position, .. } => runner_state
                    .input_event(WindowInputEvent::CursorMoved {
                        position: position.into(),
                    }),

                winit::event::WindowEvent::CursorEntered { .. } => {
                    runner_state.input_event(WindowInputEvent::CursorEntered)
                }

                winit::event::WindowEvent::CursorLeft { .. } => {
                    runner_state.input_event(WindowInputEvent::CursorLeft)
                }

                winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                    winit::event::MouseScrollDelta::LineDelta(h, v) => {
                        runner_state.input_event(WindowInputEvent::MouseWheel { delta: (h, v) })
                    }
                    winit::event::MouseScrollDelta::PixelDelta(physical_position) => runner_state
                        .input_event(WindowInputEvent::MouseWheel {
                            delta: (physical_position.x as f32, physical_position.y as f32),
                        }),
                },

                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    runner_state.input_event(WindowInputEvent::MouseInput {
                        button,
                        pressed: state.is_pressed(),
                    });
                }

                //--------------------------------------------------
                //
                winit::event::WindowEvent::RedrawRequested => runner_state.tick(event_loop),

                //--------------------------------------------------
                //
                _ => {}
            }
        }
    }

    #[inline]
    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if let Some(state) = &mut self.state {
            state.new_events(event_loop, cause);
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(state) = &mut self.state {
            state.device_event(event_loop, device_id, &event);

            match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    state.input_event(WindowInputEvent::MouseMotion { delta })
                }
                // winit::event::DeviceEvent::MouseWheel { delta } => todo!(),
                _ => {}
            }
        }
    }
}

//====================================================================
