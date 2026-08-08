use crate::Game;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const CELL_SIZE: u32 = 20;

pub struct Renderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl Renderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("browser document is unavailable"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("game canvas was not found"))?
            .dyn_into::<HtmlCanvasElement>()?;
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2D canvas is unavailable"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Self { canvas, context })
    }

    pub fn render(&self, game: &Game) {
        self.canvas.set_width(u32::from(game.width) * CELL_SIZE);
        self.canvas.set_height(u32::from(game.height) * CELL_SIZE);

        self.context.set_fill_style_str("#07130e");
        self.context.fill_rect(
            0.0,
            0.0,
            f64::from(self.canvas.width()),
            f64::from(self.canvas.height()),
        );

        self.context.set_fill_style_str("#214c37");
        for x in 0..game.width {
            self.fill_cell(x, 0);
            self.fill_cell(x, game.height - 1);
        }
        for y in 1..game.height - 1 {
            self.fill_cell(0, y);
            self.fill_cell(game.width - 1, y);
        }

        self.context.set_fill_style_str("#ffca62");
        self.fill_cell(game.food.x, game.food.y);

        for (index, segment) in game.snake.iter().enumerate() {
            let color = if index == 0 { "#d9ff6f" } else { "#74db86" };
            self.context.set_fill_style_str(color);
            self.fill_cell(segment.x, segment.y);
        }
    }

    fn fill_cell(&self, x: u16, y: u16) {
        let inset = 1.0;
        self.context.fill_rect(
            f64::from(u32::from(x) * CELL_SIZE) + inset,
            f64::from(u32::from(y) * CELL_SIZE) + inset,
            f64::from(CELL_SIZE) - inset * 2.0,
            f64::from(CELL_SIZE) - inset * 2.0,
        );
    }
}
