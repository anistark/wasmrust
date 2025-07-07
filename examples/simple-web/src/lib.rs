use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    web_sys::console::log_1(&format!("Adding {} + {}", a, b).into());
    a + b
}

#[wasm_bindgen]
pub struct Calculator {
    value: f64,
}

#[wasm_bindgen]
impl Calculator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Calculator {
        Calculator { value: 0.0 }
    }

    #[wasm_bindgen]
    pub fn add(&mut self, value: f64) {
        self.value += value;
    }

    #[wasm_bindgen]
    pub fn get_value(&self) -> f64 {
        self.value
    }
}
