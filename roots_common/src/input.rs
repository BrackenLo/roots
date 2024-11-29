//====================================================================

use std::{collections::HashSet, hash::Hash};

use crate::FastHasher;

//====================================================================

#[derive(Debug)]
pub struct Input<T> {
    pressed: HashSet<T, FastHasher>,
    just_pressed: HashSet<T, FastHasher>,
    released: HashSet<T, FastHasher>,
}

impl<T> Default for Input<T> {
    fn default() -> Self {
        Self {
            pressed: HashSet::default(),
            just_pressed: HashSet::default(),
            released: HashSet::default(),
        }
    }
}

impl<T> Input<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

#[allow(dead_code)]
impl<T> Input<T>
where
    T: Eq + Hash,
{
    #[inline]
    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    #[inline]
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    #[inline]
    pub fn released(&self, input: T) -> bool {
        self.released.contains(&input)
    }
}

pub fn process_inputs<T>(input: &mut Input<T>, val: T, pressed: bool)
where
    T: Eq + Hash + Copy,
{
    match pressed {
        true => {
            input.pressed.insert(val);
            input.just_pressed.insert(val);
        }
        false => {
            input.pressed.remove(&val);
            input.released.insert(val);
        }
    }
}

pub fn reset_input<T>(input: &mut Input<T>) {
    input.just_pressed.clear();
    input.released.clear();
}

//--------------------------------------------------

#[derive(Debug, Default)]
pub struct MouseInput {
    position: glam::Vec2,
    screen_position: glam::Vec2,
    motion_delta: glam::Vec2,
    scroll: glam::Vec2,
}

impl MouseInput {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn position(&self) -> glam::Vec2 {
        self.position
    }

    #[inline]
    pub fn screen_position(&self) -> glam::Vec2 {
        self.screen_position
    }

    #[inline]
    pub fn motion_delta(&self) -> glam::Vec2 {
        self.motion_delta
    }

    #[inline]
    pub fn scroll(&self) -> glam::Vec2 {
        self.scroll
    }
}

#[inline]
pub fn process_mouse_position(input: &mut MouseInput, position: (f64, f64)) {
    input.position = glam::vec2(position.0 as f32, position.1 as f32);
}

#[inline]
pub fn process_mouse_motion(input: &mut MouseInput, delta: (f64, f64)) {
    input.motion_delta += glam::vec2(delta.0 as f32, delta.1 as f32);
}

#[inline]
pub fn process_mouse_scroll(input: &mut MouseInput, delta: (f32, f32)) {
    input.scroll += glam::vec2(delta.0, delta.1);
}

pub fn reset_mouse_input(input: &mut MouseInput) {
    input.motion_delta = glam::Vec2::ZERO;
    input.scroll = glam::Vec2::ZERO;
}

//====================================================================
