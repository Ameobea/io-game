//! Contains definitions for the various types of entities that are spawnable into the world.

use nalgebra::{Point2, Vector2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use rustler::{types::atom::Atom, Encoder, Env, NifResult, Term};

use super::super::atoms;
use super::{Movement, COLLIDER_MARGIN};

// TODO: Move these out into config files
const BEAM_LENGTH: f32 = 50.0;
const BEAM_WIDTH: f32 = 2.0;

lazy_static! {
    pub static ref BEAM_SHAPE_HANDLE: ShapeHandle<f32> = {
        let shape = Cuboid::new(Vector2::new(
            BEAM_LENGTH / 2. - COLLIDER_MARGIN,
            BEAM_WIDTH / 2. - COLLIDER_MARGIN,
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
    Player { size: f32, movement: Movement },
    Asteroid { vertices: Vec<Point2<f32>> },
}

impl EntityType {
    pub fn to_data<'a>(&self, env: Env<'a>) -> NifResult<(Atom, Term<'a>)> {
        match self {
            EntityType::Player { size, movement } => {
                let map = Term::map_new(env);
                let map = map.map_put("size".encode(env), size.encode(env))?;
                let movement_atom: Atom = (*movement).into();
                let map = map.map_put("movement".encode(env), movement_atom.encode(env))?;

                Ok((atoms::player(), map))
            }
            EntityType::Asteroid { vertices } => {
                let map = Term::map_new(env);
                let mut mapped_verts: Vec<f32> = Vec::with_capacity(vertices.len() * 2);
                for vert in vertices {
                    mapped_verts.push(vert.x);
                    mapped_verts.push(vert.y);
                }
                let map = map.map_put("vertices".encode(env), mapped_verts.encode(env))?;

                Ok((atoms::asteroid(), map))
            }
        }
    }
}
