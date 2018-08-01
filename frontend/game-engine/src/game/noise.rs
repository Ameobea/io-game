use libcomposition::color_schemes::ColorFunction;
use libcomposition::util::build_tree_from_def;
use libcomposition::CompositionTree;
use noise::NoiseFn;

pub fn generate_background_bitmap(width: usize, height: usize) -> Vec<u8> {
    let tree_def_str = include_str!("./noise_composition_def.json");
    let (color_fn, tree): (ColorFunction, CompositionTree) =
        build_tree_from_def(tree_def_str).unwrap();

    let mut pixel_data: Vec<u8> = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let val = tree.get([x as f64, y as f64, 0.0]) as f32;
            let color = color_fn.colorize(val);
            pixel_data.push(color[0]);
            pixel_data.push(color[1]);
            pixel_data.push(color[2]);
            pixel_data.push(255u8);
        }
    }

    assert_eq!(pixel_data.len(), 1500 * 1500 * 4);

    pixel_data
}
