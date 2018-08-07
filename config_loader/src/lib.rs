#![feature(box_syntax)]

extern crate handlebars;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::Hash;
use std::io::ErrorKind;
use std::path::Path;

use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};
use serde_json::{Map, Value};

fn merge_hashmap<K: Eq + Hash, V>(mut hm1: HashMap<K, V>, mut hm2: HashMap<K, V>) -> HashMap<K, V> {
    for (k, v) in hm2.drain() {
        hm1.insert(k, v);
    }

    hm1
}

fn parse_map(map: Map<String, Value>, prefix: &str) -> HashMap<String, Value> {
    let mut keys: HashMap<String, Value> = HashMap::new();

    for (key, val) in map {
        match val {
            Value::Object(child_map) => {
                let child_keys = parse_map(child_map, &format!("{}-{}", prefix, key));
                keys = merge_hashmap(keys, child_keys);
            }
            _ => {
                keys.insert(format!("{}-{}", prefix, key), val);
            }
        }
    }

    keys
}

fn parse_config_file(json_content: &'static str) -> Map<String, Value> {
    let conf: Value = serde_json::from_str(json_content).expect("Invalid json provided!");
    if let Value::Object(obj) = conf {
        obj
    } else {
        panic!("Invalid JSON type provided to config file: {:?}", conf);
    }
}

const TEMPLATE_NAME: &'static str = "conf";

#[derive(Clone, Copy)]
struct F32Helper;

impl HelperDef for F32Helper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
        out: &mut Output,
    ) -> HelperResult {
        let val: f64 = h
            .param(0)
            .expect("No param provided to helper!")
            .value()
            .as_f64()
            .expect("Unable to parse param to `f32` helper into floating point number!");

        out.write(&format!("{:.8}", val))
            .expect("Error writing output in `f32` helper.");
        Ok(())
    }
}

pub fn build_config() {
    let out_dir = Path::new("./src/conf/");
    let out_file = out_dir.join(Path::new("mod.rs"));

    let physics_conf = parse_config_file(include_str!("../../config/physics.json"));
    let network_conf = parse_config_file(include_str!("../../config/network.json"));
    let game_conf = parse_config_file(include_str!("../../config/game.json"));

    match fs::create_dir(out_dir) {
        Ok(_) => (),
        Err(err) => match err.kind() {
            ErrorKind::AlreadyExists => (),
            _ => panic!("Unable to create `{:?}`: {:?}", out_dir, err),
        },
    }

    let output_file: File = match File::create(out_file.clone()) {
        Ok(handle) => handle,
        Err(err) => match err.kind() {
            _ => panic!("Unable to create `{:?}`: {:?}", out_file, err),
        },
    };

    let template_src = include_str!("../config.rs.hbs");
    let mut hbs = Handlebars::new();
    hbs.register_helper("f32", box F32Helper);
    hbs.register_template_string(TEMPLATE_NAME, template_src)
        .expect("Unable to register template string!");

    let mut all_configs = vec![
        parse_map(physics_conf, "physics"),
        parse_map(network_conf, "network"),
        parse_map(game_conf, "game"),
    ];
    let template_data: HashMap<String, Value> =
        all_configs.drain(..).fold(HashMap::new(), merge_hashmap);
    if let Err(err) = hbs.render_to_write(TEMPLATE_NAME, &template_data, output_file) {
        panic!(
            "Error while rendering template to output Rust source file: {:?}",
            err
        );
    }
}
