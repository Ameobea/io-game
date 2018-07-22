#![feature(use_extern_macros, wasm_custom_section, wasm_import_module)]

#[macro_use]
extern crate lazy_static;
extern crate protobuf;
extern crate uuid;
extern crate wasm_bindgen;

use std::collections::HashMap;
use std::sync::Mutex;

use wasm_bindgen::prelude::*;

pub mod util;
use self::util::{debug, log};
pub mod protos;

lazy_static! {
    static ref STATE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[wasm_bindgen]
extern "C" {
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
