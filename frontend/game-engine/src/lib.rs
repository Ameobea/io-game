#![feature(use_extern_macros, wasm_import_module)]

extern crate uuid;
extern crate wasm_bindgen;

use std::collections::HashMap;

use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

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
pub extern "C" fn greet(msg: &str) {
    let val = document.createElement("h1");
    val.set_inner_html(msg);
    document.body().append_child(val);
}
