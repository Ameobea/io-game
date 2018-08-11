//! Contains definitions for the various types of entities that are spawnable into the world.

use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d::shape::{ConvexPolygon, Cuboid, ShapeHandle};
use nphysics2d::algebra::Velocity2;
use nphysics2d::object::{BodyHandle, ColliderHandle, SensorHandle};

use super::{world::COLLIDER_MARGIN, Movement};
use conf::CONF;

pub const DEFAULT_PLAYER_SIZE: f32 = CONF.game.default_player_size;

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

pub struct EntitySpawn<T = ()> {
    pub isometry: Isometry2<f32>,
    pub velocity: Velocity2<f32>,
    pub entity: Entity,
    pub data: T,
}

pub struct EntityHandles<T> {
    pub collider_handle: ColliderHandle,
    pub body_handle: BodyHandle,
    pub beam_handle: Option<SensorHandle>,
    pub entity: Entity,
    pub data: T,
}

#[derive(Debug)]
pub struct PlayerEntity {
    pub size: u32,
    pub movement: Movement,
    pub beam_aim: Point2<f32>,
    pub beam_on: bool,
}

impl PlayerEntity {
    pub fn new(size: u32) -> Self {
        PlayerEntity {
            size,
            movement: Movement::default(),
            beam_aim: Point2::origin(),
            beam_on: false,
        }
    }
}

impl Default for PlayerEntity {
    fn default() -> Self {
        Self::new(DEFAULT_PLAYER_SIZE as u32)
    }
}

#[derive(Debug)]
pub struct AsteroidEntity {
    pub vertices: Vec<Point2<f32>>,
}

#[derive(Debug)]
pub enum Entity {
    Player(PlayerEntity),
    Asteroid(AsteroidEntity),
}

impl Entity {
    #[cfg(feature = "elixir-interop")]
    pub fn to_data<'a>(
        &self,
        env: rustler::Env<'a>,
    ) -> rustler::NifResult<(rustler::types::atom::Atom, rustler::Term<'a>)> {
        use rustler::{types::atom::Atom, Encoder, Env, NifResult, Term};

        use super::super::atoms;

        fn make_map<'a>(env: Env<'a>, items: &[(Atom, &Encoder)]) -> NifResult<Term<'a>> {
            let mut map = Term::map_new(env);
            for (key, val) in items {
                map = map.map_put(key.encode(env), val.encode(env))?;
            }
            Ok(map)
        }

        match self {
            Entity::Player(PlayerEntity {
                size,
                movement,
                beam_aim,
                beam_on,
            }) => {
                let movement_atom: Atom = (*movement).into();
                let map = make_map(
                    env,
                    &[
                        (atoms::size(), size),
                        (atoms::movement(), &movement_atom),
                        (atoms::beam_aim(), &(beam_aim.x, beam_aim.y)),
                        (atoms::beam_on(), beam_on),
                    ],
                )?;

                Ok((atoms::player(), map))
            }
            Entity::Asteroid(AsteroidEntity { vertices }) => {
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
            Entity::Player(PlayerEntity { size, .. }) => create_player_shape_handle(*size as f32),
            Entity::Asteroid(AsteroidEntity { vertices, .. }) => {
                let shape = ConvexPolygon::try_new(vertices.clone())
                    .expect("Unable to compute `ConvexPolygon` from asteroid vertices!");
                ShapeHandle::new(shape)
            }
        }
    }

    pub fn get_density(&self) -> f32 {
        match self {
            Entity::Player { .. } => 1.0,
            Entity::Asteroid { .. } => 3.5,
        }
    }
}
