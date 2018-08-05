use libcomposition::color_schemes::ColorFunction;
use libcomposition::composition::CompositionScheme;
use libcomposition::transformations::InputTransformation;
use libcomposition::{
    ComposedNoiseModule, CompositionTree, CompositionTreeNode, CompositionTreeNodeType, MasterConf,
};
use noise::{Billow, Constant, MultiFractal, NoiseFn, RidgedMulti};

fn get_composition_tree() -> CompositionTree {
    let root_node = CompositionTreeNode {
        function: CompositionTreeNodeType::Combined(ComposedNoiseModule {
            composer: CompositionScheme::Average,
            children: vec![
                CompositionTreeNode {
                    function: CompositionTreeNodeType::Leaf(box {
                        let module = RidgedMulti::new();
                        let module = module.set_frequency(1.0);
                        let module = module.set_lacunarity(2.0);
                        let module = module.set_persistence(0.5);
                        let module = module.set_attenuation(2.0);
                        module
                    }),
                    transformations: vec![InputTransformation::ZoomScale {
                        speed: 9.0,
                        zoom: 0.3,
                    }],
                },
                CompositionTreeNode {
                    function: CompositionTreeNodeType::Combined(ComposedNoiseModule {
                        composer: CompositionScheme::Average,
                        children: vec![CompositionTreeNode {
                            function: CompositionTreeNodeType::Leaf(box {
                                let module = Billow::new();
                                let module = module.set_frequency(1.0);
                                let module = module.set_lacunarity(2.0);
                                let module = module.set_persistence(0.5);
                                module
                            }),
                            transformations: vec![InputTransformation::ScaleAll(2.3)],
                        }],
                    }),
                    transformations: vec![InputTransformation::ZoomScale {
                        speed: 3.0,
                        zoom: 0.5,
                    }],
                },
                CompositionTreeNode {
                    function: CompositionTreeNodeType::Leaf(box Constant::new(-0.3)),
                    transformations: Vec::new(),
                },
            ],
        }),
        transformations: vec![InputTransformation::ZoomScale {
            speed: 0.6,
            zoom: 1.0,
        }],
    };

    CompositionTree {
        root_node,
        global_conf: MasterConf::default(),
    }
}

pub fn generate_background_texture(width: usize, height: usize) -> Vec<u8> {
    let color_fn = ColorFunction::Oceanic;
    let tree = get_composition_tree();

    let mut pixel_data: Vec<u8> = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let val = tree.get([x as f64, y as f64]) as f32;
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
