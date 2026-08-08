use crate::renderer::Renderer;
use crate::{Game, GameConfig};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Window;

struct App {
    game: Game,
    renderer: Renderer,
    tick_ms: f64,
    accumulator_ms: f64,
    last_frame_ms: Option<f64>,
}

impl App {
    fn new(config: GameConfig) -> Result<Self, JsValue> {
        Ok(Self {
            game: Game::new(config.width, config.height),
            renderer: Renderer::new("snake-canvas")?,
            tick_ms: 1_000.0 / f64::from(config.moves_per_second),
            accumulator_ms: 0.0,
            last_frame_ms: None,
        })
    }

    fn frame(&mut self, timestamp_ms: f64) {
        if let Some(last_frame_ms) = self.last_frame_ms {
            self.accumulator_ms += (timestamp_ms - last_frame_ms).min(250.0);
            while self.accumulator_ms >= self.tick_ms {
                self.game.step();
                self.accumulator_ms -= self.tick_ms;
            }
        }
        self.last_frame_ms = Some(timestamp_ms);
        self.renderer.render(&self.game);
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
