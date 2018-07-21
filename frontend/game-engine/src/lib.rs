#![feature(use_extern_macros, wasm_import_module)]

#[macro_use]
extern crate lazy_static;
extern crate uuid;
extern crate wasm_bindgen;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use uuid::Uuid;
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref STATE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

fn debug<T: Debug>(x: T) -> String {
    format!("{:?}", x)
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);

    type HTMLDocument;
    static document: HTMLDocument;
    #[wasm_bindgen(method)]
    fn createElement(this: &HTMLDocument, tagName: &str) -> Element;
    #[wasm_bindgen(method, getter)]
    fn body(this: &HTMLDocument) -> Element;

    type Element;
    #[wasm_bindgen(method, setter = innerHTML)]
    fn set_inner_html(this: &Element, html: &str);
    #[wasm_bindgen(method, js_name = appendChild)]
    fn append_child(this: &Element, other: Element);
}

#[wasm_bindgen]
pub fn greet(msg: &str) {
    let val = document.createElement("h1");
    val.set_inner_html(msg);
    document.body().append_child(val);
}

#[wasm_bindgen]
pub fn get(key: &str) -> Option<String> {
    STATE.lock().unwrap().get(key).cloned()
}

#[wasm_bindgen]
pub fn set(key: String, val: String) {
    STATE.lock().unwrap().insert(key, val);
}

#[wasm_bindgen]
pub fn handle_message(msg: &[u8]) {
    log(&debug(msg));
}
