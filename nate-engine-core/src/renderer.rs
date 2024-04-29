//!
//! Renderer Trait for Rendering Systems
//! 

use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// Rendering Engine Trait
pub trait Renderer<WORLD: Send + Sync + 'static>: Send + Sync {
    type Error: Debug;

    /// Render the necessary contents of the world
    fn render(&mut self, world: Arc<RwLock<WORLD>>) -> Result<(), Self::Error>;
}
