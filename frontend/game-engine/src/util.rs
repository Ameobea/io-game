use std::fmt::Debug;
use std::mem;

use nalgebra::Isometry2;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

lazy_static! {
    pub static ref ISOMETRY_ZERO: Isometry2<f32> = Isometry2::identity();
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn js_warn(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn js_error(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    pub fn random() -> f64;
}

pub fn debug<T: Debug>(x: T) -> String {
    format!("{:?}", x)
}

pub fn log<T: AsRef<str>>(msg: T) {
    js_log(msg.as_ref())
}

pub fn warn<T: AsRef<str>>(msg: T) {
    js_warn(msg.as_ref())
}

pub fn error<T: AsRef<str>>(msg: T) {
    js_error(msg.as_ref())
}

pub fn math_random() -> f64 {
    random()
}

/// Simulates a random UUID, but uses the rand crate with WebAssembly support.
pub fn v4_uuid() -> Uuid {
    // Because I really don't care, honestly.
    let high_quality_entropy: (f64, f64) = (math_random(), math_random());
    unsafe { mem::transmute(high_quality_entropy) }
}

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn random() -> Self {
        let (red, green, blue, _): (u8, u8, u8, [u8; 5]) = unsafe { mem::transmute(math_random()) };
        Color { red, green, blue }
    }
}
