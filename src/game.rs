use rand::Rng;
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Game {
    pub width: u16,
    pub height: u16,
    pub snake: VecDeque<Point>,
    pub direction: Direction,
    pub food: Point,
    pub score: usize,
    pub over: bool,
}

impl Game {
    pub fn new(width: u16, height: u16) -> Self {
        let mut snake = VecDeque::new();
        let start_x = width / 2;
        let start_y = height / 2;
        snake.push_back(Point {
            x: start_x,
            y: start_y,
        });
        snake.push_back(Point {
            x: start_x - 1,
            y: start_y,
        });
        let food = Self::random_food(&snake, width, height);

        Self {
            width,
            height,
            snake,
            direction: Direction::Right,
            food,
            score: 0,
            over: false,
        }
    }

    fn random_food(snake: &VecDeque<Point>, width: u16, height: u16) -> Point {
        let mut rng = rand::thread_rng();
        loop {
            let point = Point {
                x: rng.gen_range(1..(width - 1)),
                y: rng.gen_range(1..(height - 1)),
            };
            if !snake.contains(&point) {
                return point;
            }
        }
    }

    pub fn step(&mut self) {
        if self.over {
            return;
        }

        let head = self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => Point {
                x: head.x,
                y: head.y.saturating_sub(1),
            },
            Direction::Down => Point {
                x: head.x,
                y: head.y + 1,
            },
            Direction::Left => Point {
                x: head.x.saturating_sub(1),
                y: head.y,
            },
            Direction::Right => Point {
                x: head.x + 1,
                y: head.y,
            },
        };

        let eating = new_head == self.food;
        let occupied_length = self.snake.len() - usize::from(!eating);
        if new_head.x == 0
            || new_head.x == self.width - 1
            || new_head.y == 0
            || new_head.y == self.height - 1
            || self
                .snake
                .iter()
                .take(occupied_length)
                .any(|point| *point == new_head)
        {
            self.over = true;
            return;
        }

        self.snake.push_front(new_head);
        if new_head == self.food {
            self.score += 1;
            self.food = Self::random_food(&self.snake, self.width, self.height);
        } else {
            self.snake.pop_back();
        }
    }

    pub fn change_direction(&mut self, direction: Direction) {
        let reverses = matches!(
            (self.direction, direction),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        );
        if !reverses {
            self.direction = direction;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_moves_forward_without_growing() {
        let mut game = Game::new(20, 10);
        let old_head = game.snake[0];
        let old_length = game.snake.len();
        game.food = Point { x: 1, y: 1 };

        game.step();

        assert_eq!(
            game.snake[0],
            Point {
                x: old_head.x + 1,
                y: old_head.y
            }
        );
        assert_eq!(game.snake.len(), old_length);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn eating_food_grows_the_snake_and_scores() {
        let mut game = Game::new(20, 10);
        let old_length = game.snake.len();
        let head = game.snake[0];
        game.food = Point {
            x: head.x + 1,
            y: head.y,
        };

        game.step();

        assert_eq!(game.snake.len(), old_length + 1);
        assert_eq!(game.score, 1);
        assert!(!game.snake.contains(&game.food));
    }

    #[test]
    fn wall_collision_ends_the_game() {
        let mut game = Game::new(20, 10);
        game.snake = VecDeque::from([Point { x: 1, y: 4 }, Point { x: 2, y: 4 }]);
        game.direction = Direction::Left;

        game.step();

        assert!(game.over);
        assert_eq!(game.snake[0], Point { x: 1, y: 4 });
    }

    #[test]
    fn body_collision_ends_the_game() {
        let mut game = Game::new(20, 10);
        game.snake = VecDeque::from([
            Point { x: 4, y: 3 },
            Point { x: 4, y: 4 },
            Point { x: 3, y: 4 },
            Point { x: 3, y: 3 },
        ]);
        game.direction = Direction::Down;

        game.step();

        assert!(game.over);
    }

    #[test]
    fn snake_can_move_into_a_vacating_tail_cell() {
        let mut game = Game::new(20, 10);
        game.snake = VecDeque::from([
            Point { x: 4, y: 3 },
            Point { x: 4, y: 4 },
            Point { x: 3, y: 4 },
            Point { x: 3, y: 3 },
        ]);
        game.direction = Direction::Left;
        game.food = Point { x: 1, y: 1 };

        game.step();

        assert!(!game.over);
        assert_eq!(game.snake[0], Point { x: 3, y: 3 });
        assert_eq!(game.snake.len(), 4);
    }

    #[test]
    fn direction_changes_reject_immediate_reversal() {
        let mut game = Game::new(20, 10);

        game.change_direction(Direction::Left);
        assert_eq!(game.direction, Direction::Right);

        game.change_direction(Direction::Up);
        assert_eq!(game.direction, Direction::Up);
    }
}
