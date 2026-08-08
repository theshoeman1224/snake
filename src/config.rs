use std::time::Duration;

pub const MIN_WIDTH: u16 = 20;
pub const MIN_HEIGHT: u16 = 10;
pub const MAX_WIDTH: u16 = 60;
pub const MAX_HEIGHT: u16 = 40;
pub const MIN_SPEED: u16 = 2;
pub const MAX_SPEED: u16 = 20;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub moves_per_second: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_increase_speed_and_reduce_arena_size() {
        let easy = GameConfig::preset(Difficulty::Easy);
        let medium = GameConfig::preset(Difficulty::Medium);
        let hard = GameConfig::preset(Difficulty::Hard);

        assert!(easy.width > medium.width && medium.width > hard.width);
        assert!(easy.height > medium.height && medium.height > hard.height);
        assert!(easy.moves_per_second < medium.moves_per_second);
        assert!(medium.moves_per_second < hard.moves_per_second);
    }

    #[test]
    fn custom_settings_are_clamped_to_supported_limits() {
        assert_eq!(
            GameConfig::custom(1, 1, 1),
            GameConfig {
                width: MIN_WIDTH,
                height: MIN_HEIGHT,
                moves_per_second: MIN_SPEED,
            }
        );
        assert_eq!(
            GameConfig::custom(u16::MAX, u16::MAX, u16::MAX),
            GameConfig {
                width: MAX_WIDTH,
                height: MAX_HEIGHT,
                moves_per_second: MAX_SPEED,
            }
        );
    }

    #[test]
    fn tick_duration_matches_the_selected_speed() {
        let config = GameConfig::custom(30, 20, 10);
        assert_eq!(config.tick(), Duration::from_millis(100));
    }
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
