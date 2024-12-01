//====================================================================

use std::time::Duration;

use hecs::World;
use renderer::RendererState;
use roots_common::{
    input::{Input, MouseInput},
    Size, Time,
};
use roots_runner::{
    prelude::{KeyCode, MouseButton},
    window::Window,
};

pub mod renderer;
pub mod runner;
pub mod spatial;

pub use hecs;

//====================================================================

pub trait HecsApp: 'static {
    fn new(state: &mut State) -> Self
    where
        Self: Sized;

    fn resize(&mut self, state: &mut State, size: Size<u32>);
    fn tick(&mut self, state: &mut State);
}

pub struct StateOuter<A: HecsApp> {
    state: State,
    app: A,
}

pub struct State {
    pub world: World,
    pub window: Window,
    pub target_fps: Duration,

    pub renderer: RendererState,
    pub time: Time,

    pub keys: Input<KeyCode>,
    pub mouse_buttons: Input<MouseButton>,
    pub mouse_input: MouseInput,
}

impl State {
    fn new(window: Window) -> Self {
        let world = World::new();

        let renderer = RendererState::new(&window);

        let time = Time::new();

        State {
            world,
            window,
            renderer,
            target_fps: Duration::from_secs_f32(1. / 75.),
            time,
            keys: Input::new(),
            mouse_buttons: Input::new(),
            mouse_input: MouseInput::new(),
        }
    }
}

//====================================================================

//====================================================================
