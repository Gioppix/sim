pub mod ant;
pub mod constants;
pub mod coordinates;
pub mod food;
pub mod snapshot;
pub mod world;

pub use ant::{Action, Ant, Memory};
pub use coordinates::Coordinates;
pub use food::FoodItem;
pub use snapshot::{AntSnapshot, FoodSnapshot, WorldMetadata, WorldSnapshot, metadata, snapshot};
pub use world::{RestrictedWorld, World, WorldConfig};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod evaluation;
