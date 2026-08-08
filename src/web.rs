use crate::renderer::Renderer;
use crate::Game;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let game = Game::new(40, 22);
    Renderer::new("snake-canvas")?.render(&game);
    Ok(())
}
