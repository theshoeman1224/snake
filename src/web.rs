use crate::renderer::Renderer;
use crate::{Direction, Game, GameConfig};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Event, KeyboardEvent, Window};

struct App {
    game: Game,
    renderer: Renderer,
    tick_ms: f64,
    accumulator_ms: f64,
    last_frame_ms: Option<f64>,
    pending_direction: Option<Direction>,
    paused: bool,
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
        })
    }

    fn frame(&mut self, timestamp_ms: f64) {
        if self.paused {
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
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let config = GameConfig {
        width: 40,
        height: 22,
        moves_per_second: 8,
    };
    let app = Rc::new(RefCell::new(App::new(config)?));
    install_controls(Rc::clone(&app))?;
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
