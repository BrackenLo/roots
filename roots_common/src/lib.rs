//====================================================================

use std::{
    fmt::Display,
    hash::BuildHasherDefault,
    ops::{Deref, DerefMut},
};

use rustc_hash::FxHasher;
use web_time::{Duration, Instant};

pub mod input;
pub mod spatial;

//====================================================================

pub type FastHasher = BuildHasherDefault<FxHasher>;

//====================================================================

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[allow(dead_code)]
impl<T> Size<T> {
    #[inline]
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl<T> From<(T, T)> for Size<T> {
    #[inline]
    fn from(value: (T, T)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl<T> From<Size<T>> for (T, T) {
    #[inline]
    fn from(value: Size<T>) -> Self {
        (value.width, value.height)
    }
}

impl<T: Display> Display for Size<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.width, self.height)
    }
}

//====================================================================

#[derive(Debug)]
pub struct Time {
    elapsed: Instant,

    last_frame: Instant,
    delta: Duration,
    delta_seconds: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            elapsed: Instant::now(),
            last_frame: Instant::now(),
            delta: Duration::ZERO,
            delta_seconds: 0.,
        }
    }
}

impl Time {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn elapsed(&self) -> &Instant {
        &self.elapsed
    }

    #[inline]
    pub fn delta(&self) -> &Duration {
        &self.delta
    }

    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }
}

pub fn tick_time(time: &mut Time) {
    time.delta = time.last_frame.elapsed();
    time.delta_seconds = time.delta.as_secs_f32();

    time.last_frame = Instant::now();
}

//====================================================================

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct WasmWrapper<T>(T);

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct WasmWrapper<T>(send_wrapper::SendWrapper<T>);

impl<T> WasmWrapper<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        return Self(data);

        #[cfg(target_arch = "wasm32")]
        return Self(send_wrapper::SendWrapper::new(data));
    }

    #[inline]
    pub fn take(self) -> T {
        #[cfg(not(target_arch = "wasm32"))]
        return self.0;

        #[cfg(target_arch = "wasm32")]
        return self.0.take();
    }
}

impl<T> Deref for WasmWrapper<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WasmWrapper<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

//====================================================================
