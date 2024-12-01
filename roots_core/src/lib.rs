//====================================================================

pub use roots_common as common;
#[cfg(feature = "hecs")]
pub use roots_hecs as hecs;
pub use roots_pipelines as pipelines;
pub use roots_renderer as renderer;
pub use roots_runner as runner;
pub use roots_text as text;

//====================================================================

pub mod prelude {
    pub use roots_common::{Size, Time};
    #[cfg(feature = "hecs")]
    pub use roots_hecs::State;
    pub use roots_renderer::{camera, Color, Device, Queue, Surface, SurfaceConfig};
}

//====================================================================
