use std::fmt::Debug;

use wasm_bindgen::prelude::*;

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
