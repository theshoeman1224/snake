use std::time::Duration;

pub const MIN_WIDTH: u16 = 20;
pub const MIN_HEIGHT: u16 = 10;

#[derive(Copy, Clone)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub moves_per_second: u16,
}

impl GameConfig {
    pub fn tick(self) -> Duration {
        Duration::from_millis(1_000 / u64::from(self.moves_per_second))
    }
}
