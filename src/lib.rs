pub mod config;
pub mod game;

#[cfg(target_arch = "wasm32")]
mod renderer;
#[cfg(target_arch = "wasm32")]
mod web;

pub use config::{Difficulty, GameConfig};
pub use game::{Direction, Game, Point};

pub const GIT_COMMIT: &str = env!("SNAKE_GIT_SHA");
