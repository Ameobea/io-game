//! Contains definitions for the various types of entities that are spawnable into the world.

use nalgebra::{Point2, Vector2};
use ncollide2d::shape::{ConvexPolygon, Cuboid, ShapeHandle};
use rustler::{types::atom::Atom, Encoder, Env, NifResult, Term};

use super::super::atoms;
use super::{Movement, COLLIDER_MARGIN};
use conf::CONF;

lazy_static! {
    pub static ref BEAM_SHAPE_HANDLE: ShapeHandle<f32> = {
        let shape = Cuboid::new(Vector2::new(
            CONF.game.player_beam_length / 2. - COLLIDER_MARGIN,
            CONF.game.player_beam_width / 2. - COLLIDER_MARGIN,
        ));
        ShapeHandle::new(shape)
    };
}

pub fn create_player_shape_handle(size: f32) -> ShapeHandle<f32> {
    let shape = Cuboid::new(Vector2::new(
        size / 2. - COLLIDER_MARGIN,
        size / 2. - COLLIDER_MARGIN,
    ));
    ShapeHandle::new(shape)
}

pub enum EntityType {
    Player {
        size: f32,
        movement: Movement,
        beam_aim: f32,
        beam_on: bool,
    },
    Asteroid {
        vertices: Vec<Point2<f32>>,
    },
}

fn make_map<'a>(env: Env<'a>, items: &[(Atom, &Encoder)]) -> NifResult<Term<'a>> {
    let mut map = Term::map_new(env);
    for (key, val) in items {
        map = map.map_put(key.encode(env), val.encode(env))?;
    }
    Ok(map)
}

impl EntityType {
    pub fn to_data<'a>(&self, env: Env<'a>) -> NifResult<(Atom, Term<'a>)> {
        match self {
            EntityType::Player {
                size,
                movement,
                beam_aim,
                beam_on,
            } => {
                let movement_atom: Atom = (*movement).into();
                let map = make_map(
                    env,
                    &[
                        (atoms::size(), size),
                        (atoms::movement(), &movement_atom),
                        (atoms::beam_aim(), beam_aim),
                        (atoms::beam_on(), beam_on),
                    ],
                )?;

                Ok((atoms::player(), map))
            }
            EntityType::Asteroid { vertices } => {
                let map = Term::map_new(env);
                let mut mapped_verts: Vec<f32> = Vec::with_capacity(vertices.len() * 2);
                for vert in vertices {
                    mapped_verts.push(vert.x);
                    mapped_verts.push(vert.y);
                }
                let map =
                    map.map_put(atoms::vert_coords().encode(env), mapped_verts.encode(env))?;

                Ok((atoms::asteroid(), map))
            }
        }
    }

    pub fn get_shape_handle(&self) -> ShapeHandle<f32> {
        match self {
            EntityType::Player { size, .. } => create_player_shape_handle(*size),
            EntityType::Asteroid { vertices, .. } => {
                let shape = ConvexPolygon::try_new(vertices.clone())
                    .expect("Unable to compute `ConvexPolygon` from asteroid vertices!");
                ShapeHandle::new(shape)
            }
        }
    }

    pub fn get_density(&self) -> f32 {
        match self {
            EntityType::Player { .. } => 1.0,
            EntityType::Asteroid { .. } => 3.5,
        }
    }
}
