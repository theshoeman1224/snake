use crate::renderer::Renderer;
use crate::{Difficulty, Direction, Game, GameConfig};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Event, HtmlInputElement, KeyboardEvent, Window};

struct App {
    game: Game,
    renderer: Renderer,
    tick_ms: f64,
    accumulator_ms: f64,
    last_frame_ms: Option<f64>,
    pending_direction: Option<Direction>,
    paused: bool,
    running: bool,
}

impl App {
    fn new(config: GameConfig) -> Result<Self, JsValue> {
        Ok(Self {
            game: Game::new(config.width, config.height),
            renderer: Renderer::new("snake-canvas")?,
            tick_ms: 1_000.0 / f64::from(config.moves_per_second),
            accumulator_ms: 0.0,
            last_frame_ms: None,
            pending_direction: None,
            paused: false,
            running: false,
        })
    }

    fn frame(&mut self, timestamp_ms: f64) {
        if self.paused || !self.running {
            self.last_frame_ms = None;
            self.renderer.render(&self.game);
            return;
        }

        if let Some(last_frame_ms) = self.last_frame_ms {
            self.accumulator_ms += (timestamp_ms - last_frame_ms).min(250.0);
            while self.accumulator_ms >= self.tick_ms {
                if let Some(direction) = self.pending_direction.take() {
                    self.game.change_direction(direction);
                }
                self.game.step();
                self.accumulator_ms -= self.tick_ms;
            }
        }
        self.last_frame_ms = Some(timestamp_ms);
        self.renderer.render(&self.game);
    }

    fn queue_direction(&mut self, direction: Direction) {
        if self.pending_direction.is_none() {
            self.pending_direction = Some(direction);
        }
    }

    fn restart(&mut self, config: GameConfig) {
        self.game = Game::new(config.width, config.height);
        self.tick_ms = 1_000.0 / f64::from(config.moves_per_second);
        self.accumulator_ms = 0.0;
        self.last_frame_ms = None;
        self.pending_direction = None;
        self.running = true;
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let config = GameConfig::preset(Difficulty::Medium);
    let app = Rc::new(RefCell::new(App::new(config)?));
    install_controls(Rc::clone(&app))?;
    install_mode_controls(Rc::clone(&app))?;
    let animation_callback = Rc::new(RefCell::new(None));
    let callback_handle = Rc::clone(&animation_callback);
    let app_handle = Rc::clone(&app);

    *animation_callback.borrow_mut() = Some(Closure::new(move |timestamp_ms: f64| {
        app_handle.borrow_mut().frame(timestamp_ms);
        let callback = callback_handle.borrow();
        request_frame(callback.as_ref().unwrap()).expect("animation frame should be scheduled");
    }));

    request_frame(animation_callback.borrow().as_ref().unwrap())?;
    Ok(())
}

fn request_frame(callback: &Closure<dyn FnMut(f64)>) -> Result<i32, JsValue> {
    browser_window()?.request_animation_frame(callback.as_ref().unchecked_ref())
}

fn browser_window() -> Result<Window, JsValue> {
    web_sys::window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))
}

fn browser_document() -> Result<Document, JsValue> {
    browser_window()?
        .document()
        .ok_or_else(|| JsValue::from_str("browser document is unavailable"))
}

fn install_controls(app: Rc<RefCell<App>>) -> Result<(), JsValue> {
    let window = browser_window()?;
    let keyboard_app = Rc::clone(&app);
    let keyboard = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
        let direction = match event.key().as_str() {
            "ArrowUp" | "w" | "W" => Some(Direction::Up),
            "ArrowDown" | "s" | "S" => Some(Direction::Down),
            "ArrowLeft" | "a" | "A" => Some(Direction::Left),
            "ArrowRight" | "d" | "D" => Some(Direction::Right),
            _ => None,
        };
        if let Some(direction) = direction {
            event.prevent_default();
            keyboard_app.borrow_mut().queue_direction(direction);
        }
    });
    window.add_event_listener_with_callback("keydown", keyboard.as_ref().unchecked_ref())?;
    keyboard.forget();

    let document = browser_document()?;
    for (id, direction) in [
        ("control-up", Direction::Up),
        ("control-down", Direction::Down),
        ("control-left", Direction::Left),
        ("control-right", Direction::Right),
    ] {
        let element = document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str("touch control was not found"))?;
        let control_app = Rc::clone(&app);
        let control = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            event.prevent_default();
            control_app.borrow_mut().queue_direction(direction);
        });
        element
            .add_event_listener_with_callback("pointerdown", control.as_ref().unchecked_ref())?;
        control.forget();
    }

    let blur_app = Rc::clone(&app);
    let blur = Closure::<dyn FnMut(Event)>::new(move |_| blur_app.borrow_mut().paused = true);
    window.add_event_listener_with_callback("blur", blur.as_ref().unchecked_ref())?;
    blur.forget();

    let focus = Closure::<dyn FnMut(Event)>::new(move |_| app.borrow_mut().paused = false);
    window.add_event_listener_with_callback("focus", focus.as_ref().unchecked_ref())?;
    focus.forget();
    Ok(())
}

fn install_mode_controls(app: Rc<RefCell<App>>) -> Result<(), JsValue> {
    let document = browser_document()?;
    for (id, difficulty) in [
        ("mode-easy", Difficulty::Easy),
        ("mode-medium", Difficulty::Medium),
        ("mode-hard", Difficulty::Hard),
    ] {
        let button = document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str("mode button was not found"))?;
        let mode_app = Rc::clone(&app);
        let select_mode = Closure::<dyn FnMut(Event)>::new(move |_| {
            mode_app
                .borrow_mut()
                .restart(GameConfig::preset(difficulty));
        });
        button.add_event_listener_with_callback("click", select_mode.as_ref().unchecked_ref())?;
        select_mode.forget();
    }

    let width = custom_input(&document, "custom-width")?;
    let height = custom_input(&document, "custom-height")?;
    let speed = custom_input(&document, "custom-speed")?;
    let custom_button = document
        .get_element_by_id("mode-custom")
        .ok_or_else(|| JsValue::from_str("custom mode button was not found"))?;
    let select_custom = Closure::<dyn FnMut(Event)>::new(move |_| {
        let config = GameConfig::custom(
            width.value_as_number() as u16,
            height.value_as_number() as u16,
            speed.value_as_number() as u16,
        );
        app.borrow_mut().restart(config);
    });
    custom_button
        .add_event_listener_with_callback("click", select_custom.as_ref().unchecked_ref())?;
    select_custom.forget();
    Ok(())
}

fn custom_input(document: &Document, id: &str) -> Result<HtmlInputElement, JsValue> {
    Ok(document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str("custom setting was not found"))?
        .dyn_into::<HtmlInputElement>()?)
}
