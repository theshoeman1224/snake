use crate::renderer::Renderer;
use crate::{Difficulty, Direction, Game, GameConfig, GIT_COMMIT};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Event, HtmlInputElement, KeyboardEvent, Window};

struct App {
    game: Game,
    renderer: Renderer,
    config: GameConfig,
    tick_ms: f64,
    accumulator_ms: f64,
    last_frame_ms: Option<f64>,
    pending_direction: Option<Direction>,
    manual_paused: bool,
    focus_paused: bool,
    running: bool,
    score: Element,
    status: Element,
    overlay: Element,
    pause_button: Element,
}

impl App {
    fn new(config: GameConfig) -> Result<Self, JsValue> {
        Ok(Self {
            game: Game::new(config.width, config.height),
            renderer: Renderer::new("snake-canvas")?,
            config,
            tick_ms: 1_000.0 / f64::from(config.moves_per_second),
            accumulator_ms: 0.0,
            last_frame_ms: None,
            pending_direction: None,
            manual_paused: false,
            focus_paused: false,
            running: false,
            score: required_element("score")?,
            status: required_element("game-status")?,
            overlay: required_element("game-overlay")?,
            pause_button: required_element("pause-game")?,
        })
    }

    fn frame(&mut self, timestamp_ms: f64) {
        if self.manual_paused || self.focus_paused || !self.running {
            self.last_frame_ms = None;
            self.renderer.render(&self.game);
            self.update_interface();
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
        self.update_interface();
    }

    fn queue_direction(&mut self, direction: Direction) {
        if self.pending_direction.is_none() {
            self.pending_direction = Some(direction);
        }
    }

    fn restart(&mut self, config: GameConfig) {
        self.game = Game::new(config.width, config.height);
        self.config = config;
        self.tick_ms = 1_000.0 / f64::from(config.moves_per_second);
        self.accumulator_ms = 0.0;
        self.last_frame_ms = None;
        self.pending_direction = None;
        self.manual_paused = false;
        self.running = true;
    }

    fn update_interface(&self) {
        self.score
            .set_text_content(Some(&self.game.score.to_string()));
        let status = if !self.running {
            "Choose a mode"
        } else if self.game.over {
            "Game over"
        } else if self.manual_paused || self.focus_paused {
            "Paused"
        } else {
            "Playing"
        };
        self.status.set_text_content(Some(status));
        self.pause_button
            .set_text_content(Some(if self.manual_paused {
                "Resume"
            } else {
                "Pause"
            }));
        if self.game.over {
            let _ = self.overlay.remove_attribute("hidden");
        } else {
            let _ = self.overlay.set_attribute("hidden", "");
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    display_build_commit()?;
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

fn display_build_commit() -> Result<(), JsValue> {
    let short_commit = GIT_COMMIT.get(..8).unwrap_or(GIT_COMMIT);
    required_element("build-commit")?.set_text_content(Some(short_commit));
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

fn required_element(id: &str) -> Result<Element, JsValue> {
    browser_document()?
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("required element #{id} was not found")))
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
    let blur = Closure::<dyn FnMut(Event)>::new(move |_| blur_app.borrow_mut().focus_paused = true);
    window.add_event_listener_with_callback("blur", blur.as_ref().unchecked_ref())?;
    blur.forget();

    let focus = Closure::<dyn FnMut(Event)>::new(move |_| app.borrow_mut().focus_paused = false);
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
    let custom_app = Rc::clone(&app);
    let select_custom = Closure::<dyn FnMut(Event)>::new(move |_| {
        let config = GameConfig::custom(
            width.value_as_number() as u16,
            height.value_as_number() as u16,
            speed.value_as_number() as u16,
        );
        custom_app.borrow_mut().restart(config);
    });
    custom_button
        .add_event_listener_with_callback("click", select_custom.as_ref().unchecked_ref())?;
    select_custom.forget();

    let pause_button = required_element("pause-game")?;
    let pause_app = Rc::clone(&app);
    let toggle_pause = Closure::<dyn FnMut(Event)>::new(move |_| {
        let mut app = pause_app.borrow_mut();
        if app.running && !app.game.over {
            app.manual_paused = !app.manual_paused;
        }
    });
    pause_button
        .add_event_listener_with_callback("click", toggle_pause.as_ref().unchecked_ref())?;
    toggle_pause.forget();

    for id in ["restart-game", "restart-overlay"] {
        let restart_button = required_element(id)?;
        let restart_app = Rc::clone(&app);
        let restart = Closure::<dyn FnMut(Event)>::new(move |_| {
            let config = restart_app.borrow().config;
            restart_app.borrow_mut().restart(config);
        });
        restart_button
            .add_event_listener_with_callback("click", restart.as_ref().unchecked_ref())?;
        restart.forget();
    }
    Ok(())
}

fn custom_input(document: &Document, id: &str) -> Result<HtmlInputElement, JsValue> {
    Ok(document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str("custom setting was not found"))?
        .dyn_into::<HtmlInputElement>()?)
}
