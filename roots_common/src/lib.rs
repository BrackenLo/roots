//====================================================================

use std::fmt::Display;

pub mod spatial;

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
