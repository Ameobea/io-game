#![feature(use_extern_macros, wasm_import_module)]

extern crate uuid;
extern crate wasm_bindgen;

use std::collections::HashMap;

use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

pub struct Message {
    pub id: Uuid,
    pub message: String,
}

#[wasm_bindgen]
pub fn greet(msg: &Message) {
    let &Message { name, id } = msg;
    let mut hm: HashMap<&'static str, &'static str> = HashMap::new();
    hm.insert("test", "test1").unwrap();
    hm.insert("test2", "test3").unwrap();
    alert(&format!("Hello, {} {} {}!", hm.get("test").unwrap(), name, id));
}
