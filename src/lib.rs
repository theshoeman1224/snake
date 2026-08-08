pub mod config;
pub mod game;

#[cfg(target_arch = "wasm32")]
mod web;

pub use config::GameConfig;
pub use game::{Direction, Game, Point};
