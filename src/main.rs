use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, BeginSynchronizedUpdate, Clear, ClearType,
    EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use rand::Rng;
use std::collections::VecDeque;
use std::io::{self, stdout, Write};
use std::time::{Duration, Instant};

const MIN_WIDTH: u16 = 20;
const MIN_HEIGHT: u16 = 10;

#[derive(Copy, Clone, PartialEq, Eq)]
struct Point {
    x: u16,
    y: u16,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Game {
    width: u16,
    height: u16,
    snake: VecDeque<Point>,
    dir: Direction,
    food: Point,
    score: usize,
    over: bool,
}

#[derive(Copy, Clone)]
struct GameConfig {
    width: u16,
    height: u16,
    moves_per_second: u16,
}

impl GameConfig {
    fn tick(self) -> Duration {
        Duration::from_millis(1_000 / u64::from(self.moves_per_second))
    }
}

impl Game {
    fn new(width: u16, height: u16) -> Self {
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
        let food = Game::random_food(&snake, width, height);
        Game {
            width,
            height,
            snake,
            dir: Direction::Right,
            food,
            score: 0,
            over: false,
        }
    }

    fn random_food(snake: &VecDeque<Point>, width: u16, height: u16) -> Point {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(1..(width - 1));
            let y = rng.gen_range(1..(height - 1));
            let p = Point { x, y };
            if !snake.iter().any(|s| s.x == p.x && s.y == p.y) {
                return p;
            }
        }
    }

    fn step(&mut self) {
        if self.over {
            return;
        }
        let head = self.snake.front().unwrap();
        let new_head = match self.dir {
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

        // collisions with walls
        if new_head.x == 0
            || new_head.x == self.width - 1
            || new_head.y == 0
            || new_head.y == self.height - 1
        {
            self.over = true;
            return;
        }

        // collisions with self
        if self
            .snake
            .iter()
            .any(|p| p.x == new_head.x && p.y == new_head.y)
        {
            self.over = true;
            return;
        }

        self.snake.push_front(new_head);

        if new_head.x == self.food.x && new_head.y == self.food.y {
            self.score += 1;
            self.food = Game::random_food(&self.snake, self.width, self.height);
        } else {
            self.snake.pop_back();
        }
    }
}

fn preset_config(mode: usize, max_width: u16, max_height: u16) -> GameConfig {
    let (width, height, moves_per_second) = match mode {
        0 => (80, 40, 5),
        1 => (60, 28, 8),
        2 => (40, 18, 12),
        _ => unreachable!(),
    };

    GameConfig {
        width: width.min(max_width),
        height: height.min(max_height),
        moves_per_second,
    }
}

fn draw_mode_menu<W: Write>(
    out: &mut W,
    selected: usize,
    max_width: u16,
    max_height: u16,
) -> io::Result<()> {
    queue!(
        out,
        BeginSynchronizedUpdate,
        MoveTo(0, 0),
        Clear(ClearType::All),
        Print("SNAKE\r\n\r\n"),
        Print("Select a game mode with Up/Down and Enter:\r\n\r\n")
    )?;

    for (index, name) in ["Easy", "Medium", "Hard", "Custom"].iter().enumerate() {
        let marker = if selected == index { ">" } else { " " };
        if index < 3 {
            let config = preset_config(index, max_width, max_height);
            queue!(
                out,
                Print(format!(
                    " {marker} {name:<6}  {:>2} moves/s  {}x{} arena\r\n",
                    config.moves_per_second, config.width, config.height
                ))
            )?;
        } else {
            queue!(
                out,
                Print(format!(
                    " {marker} {name:<6}  Choose speed and arena size\r\n"
                ))
            )?;
        }
    }

    queue!(
        out,
        Print("\r\nPress 1-4 for quick selection, or q to quit."),
        EndSynchronizedUpdate
    )?;
    out.flush()
}

fn draw_custom_menu<W: Write>(out: &mut W, config: GameConfig, selected: usize) -> io::Result<()> {
    queue!(
        out,
        BeginSynchronizedUpdate,
        MoveTo(0, 0),
        Clear(ClearType::All),
        Print("CUSTOM MODE\r\n\r\n"),
        Print("Use Up/Down to select and Left/Right to adjust:\r\n\r\n")
    )?;

    let settings = [
        ("Arena width", config.width.to_string()),
        ("Arena height", config.height.to_string()),
        (
            "Snake speed",
            format!("{} moves/s", config.moves_per_second),
        ),
    ];
    for (index, (name, value)) in settings.iter().enumerate() {
        let marker = if selected == index { ">" } else { " " };
        queue!(out, Print(format!(" {marker} {name:<14} {value}\r\n")))?;
    }

    queue!(
        out,
        Print("\r\nPress Enter to play or Esc to return."),
        EndSynchronizedUpdate
    )?;
    out.flush()
}

fn select_custom_config<W: Write>(
    out: &mut W,
    max_width: u16,
    max_height: u16,
) -> io::Result<Option<GameConfig>> {
    let mut config = preset_config(1, max_width, max_height);
    let mut selected = 0;

    loop {
        draw_custom_menu(out, config, selected)?;
        if let Event::Key(KeyEvent { code, .. }) = read()? {
            match code {
                KeyCode::Up => selected = selected.saturating_sub(1),
                KeyCode::Down => selected = (selected + 1).min(2),
                KeyCode::Left => match selected {
                    0 => config.width = config.width.saturating_sub(1).max(MIN_WIDTH),
                    1 => config.height = config.height.saturating_sub(1).max(MIN_HEIGHT),
                    2 => config.moves_per_second = config.moves_per_second.saturating_sub(1).max(2),
                    _ => unreachable!(),
                },
                KeyCode::Right => match selected {
                    0 => config.width = (config.width + 1).min(max_width),
                    1 => config.height = (config.height + 1).min(max_height),
                    2 => config.moves_per_second = (config.moves_per_second + 1).min(20),
                    _ => unreachable!(),
                },
                KeyCode::Enter => return Ok(Some(config)),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                _ => {}
            }
        }
    }
}

fn select_config<W: Write>(
    out: &mut W,
    max_width: u16,
    max_height: u16,
) -> io::Result<Option<GameConfig>> {
    let mut selected = 1;

    loop {
        draw_mode_menu(out, selected, max_width, max_height)?;
        if let Event::Key(KeyEvent { code, .. }) = read()? {
            match code {
                KeyCode::Up => selected = selected.saturating_sub(1),
                KeyCode::Down => selected = (selected + 1).min(3),
                KeyCode::Char(number @ '1'..='4') => {
                    selected = number.to_digit(10).unwrap() as usize - 1;
                    if selected < 3 {
                        return Ok(Some(preset_config(selected, max_width, max_height)));
                    }
                    if let Some(config) = select_custom_config(out, max_width, max_height)? {
                        return Ok(Some(config));
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                KeyCode::Enter => {
                    if selected < 3 {
                        return Ok(Some(preset_config(selected, max_width, max_height)));
                    }
                    if let Some(config) = select_custom_config(out, max_width, max_height)? {
                        return Ok(Some(config));
                    }
                }
                _ => {}
            }
        }
    }
}

fn draw_board<W: Write>(out: &mut W, game: &Game) -> io::Result<()> {
    let border = "#".repeat(game.width.into());
    queue!(
        out,
        BeginSynchronizedUpdate,
        MoveTo(0, 0),
        Clear(ClearType::All),
        Print(&border),
        MoveTo(0, game.height - 1),
        Print(&border)
    )?;

    for y in 1..game.height - 1 {
        queue!(
            out,
            MoveTo(0, y),
            Print("#"),
            MoveTo(game.width - 1, y),
            Print("#")
        )?;
    }

    queue!(out, MoveTo(game.food.x, game.food.y), Print("*"))?;
    for (index, point) in game.snake.iter().enumerate() {
        let segment = if index == 0 { "O" } else { "o" };
        queue!(out, MoveTo(point.x, point.y), Print(segment))?;
    }

    queue!(
        out,
        MoveTo(0, game.height),
        Print(format!(" Score: {}  (Press 'q' to quit)", game.score)),
        EndSynchronizedUpdate
    )?;

    out.flush()
}

fn draw_step<W: Write>(
    out: &mut W,
    game: &Game,
    old_head: Point,
    old_tail: Point,
    old_food: Point,
    old_score: usize,
) -> io::Result<()> {
    if game.over {
        return Ok(());
    }

    let new_head = *game.snake.front().unwrap();
    let grew = game.score != old_score;

    queue!(
        out,
        BeginSynchronizedUpdate,
        MoveTo(old_head.x, old_head.y),
        Print("o")
    )?;

    if !grew {
        queue!(out, MoveTo(old_tail.x, old_tail.y), Print(" "))?;
    } else {
        queue!(
            out,
            MoveTo(game.food.x, game.food.y),
            Print("*"),
            MoveTo(0, game.height),
            Print(format!(" Score: {}  (Press 'q' to quit)", game.score))
        )?;
    }

    debug_assert!(grew || game.food == old_food);
    queue!(
        out,
        MoveTo(new_head.x, new_head.y),
        Print("O"),
        EndSynchronizedUpdate
    )?;

    out.flush()
}

fn run_game<W: Write>(stdout: &mut W, config: GameConfig) -> io::Result<()> {
    let mut game = Game::new(config.width, config.height);
    let tick = config.tick();
    let mut last_tick = Instant::now();

    draw_board(stdout, &game)?;

    'outer: loop {
        // input
        if poll(Duration::from_millis(1_0))? {
            if let Event::Key(KeyEvent { code, .. }) = read()? {
                match code {
                    KeyCode::Char('q') => break 'outer,
                    KeyCode::Up => {
                        if let Direction::Down = game.dir {
                        } else {
                            game.dir = Direction::Up
                        }
                    }
                    KeyCode::Down => {
                        if let Direction::Up = game.dir {
                        } else {
                            game.dir = Direction::Down
                        }
                    }
                    KeyCode::Left => {
                        if let Direction::Right = game.dir {
                        } else {
                            game.dir = Direction::Left
                        }
                    }
                    KeyCode::Right => {
                        if let Direction::Left = game.dir {
                        } else {
                            game.dir = Direction::Right
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick {
            let old_head = *game.snake.front().unwrap();
            let old_tail = *game.snake.back().unwrap();
            let old_food = game.food;
            let old_score = game.score;
            game.step();
            draw_step(stdout, &game, old_head, old_tail, old_food, old_score)?;
            last_tick += tick;
        }

        if game.over {
            queue!(
                stdout,
                BeginSynchronizedUpdate,
                MoveTo(0, game.height + 1),
                Print(format!(
                    "Game Over! Final score: {}. Press 'q' to exit.",
                    game.score
                )),
                EndSynchronizedUpdate
            )?;
            stdout.flush()?;
            // wait for q
            loop {
                if poll(Duration::from_millis(100))? {
                    if let Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }) = read()?
                    {
                        break 'outer;
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let (terminal_width, terminal_height) = crossterm::terminal::size()?;
    if terminal_width < MIN_WIDTH || terminal_height < MIN_HEIGHT + 2 {
        eprintln!("Terminal must be at least 20 columns by 12 rows to play Snake.");
        return Ok(());
    }

    let max_width = terminal_width;
    let max_height = terminal_height - 2;
    let mut stdout = stdout();

    enable_raw_mode()?;
    if let Err(error) = execute!(stdout, EnterAlternateScreen, Hide) {
        let _ = disable_raw_mode();
        return Err(error);
    }

    let game_result = match select_config(&mut stdout, max_width, max_height) {
        Ok(Some(config)) => run_game(&mut stdout, config),
        Ok(None) => Ok(()),
        Err(error) => Err(error),
    };
    let screen_result = execute!(stdout, Show, LeaveAlternateScreen);
    let raw_mode_result = disable_raw_mode();

    game_result.and(screen_result).and(raw_mode_result)
}
