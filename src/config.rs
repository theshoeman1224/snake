use std::time::Duration;

pub const MIN_WIDTH: u16 = 20;
pub const MIN_HEIGHT: u16 = 10;
pub const MAX_WIDTH: u16 = 60;
pub const MAX_HEIGHT: u16 = 40;
pub const MIN_SPEED: u16 = 2;
pub const MAX_SPEED: u16 = 20;

#[derive(Copy, Clone)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

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

    pub fn preset(difficulty: Difficulty) -> Self {
        match difficulty {
            Difficulty::Easy => Self {
                width: 36,
                height: 24,
                moves_per_second: 5,
            },
            Difficulty::Medium => Self {
                width: 30,
                height: 20,
                moves_per_second: 8,
            },
            Difficulty::Hard => Self {
                width: 24,
                height: 16,
                moves_per_second: 12,
            },
        }
    }

    pub fn custom(width: u16, height: u16, moves_per_second: u16) -> Self {
        Self {
            width: width.clamp(MIN_WIDTH, MAX_WIDTH),
            height: height.clamp(MIN_HEIGHT, MAX_HEIGHT),
            moves_per_second: moves_per_second.clamp(MIN_SPEED, MAX_SPEED),
        }
    }
}
