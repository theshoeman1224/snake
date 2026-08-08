use rand::Rng;
use std::collections::VecDeque;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

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

        if new_head.x == 0
            || new_head.x == self.width - 1
            || new_head.y == 0
            || new_head.y == self.height - 1
            || self.snake.contains(&new_head)
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
}
