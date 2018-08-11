use std::hint::unreachable_unchecked;

use nalgebra::{Isometry2, Point2, Vector2};
pub use native_physics::physics::entities::{
    AsteroidEntity, Entity, EntityHandles, EntitySpawn, PlayerEntity,
};
use ncollide2d::query::Ray;
use nphysics2d::algebra::Velocity2;

use game::effects::DrillingParticles;
use game_state::{get_effects_manager, get_state};
use physics_math::ray_collision;
use proto_utils::ServerMessageContent;
use protos::server_messages::{CreationEvent, CreationEvent_oneof_entity as ProtoEntity};
use render_methods::{fill_poly, render_line};
use util::{error, Color};

/// An optional piece of client-local state attached to an entity for things such as visual
/// appearance and transitive state not transmitted authoritatively by the server.
#[derive(Debug)]
pub enum ClientState {
    Player {
        color: Color,
        vertices: Vec<Point2<f32>>,
    },
    Asteroid {
        color: Color,
    },
}

pub fn apply_update(
    _entity: &mut Entity,
    _client_state: &mut ClientState,
    _update: &ServerMessageContent,
) {
    // TODO
}

pub fn tick(_entity: &mut Entity, _client_state: &mut ClientState, _cur_tick: usize) {}

fn transform_points(pts: &[Point2<f32>], isometry: &Isometry2<f32>) -> Vec<f32> {
    let mut buf = Vec::with_capacity(pts.len() * 2);
    for pt in pts {
        let transformed_pt = isometry * pt;
        buf.push(transformed_pt.x);
        buf.push(transformed_pt.y);
    }
    buf
}

fn player_verts(size: f32) -> [Point2<f32>; 4] {
    let half = size / 2.0;
    [
        Point2::new(-half, half),
        Point2::new(-half, -half),
        Point2::new(half, -half),
        Point2::new(half, half),
    ]
}

fn unmatched_state(entity: &Entity, client_state: &ClientState) -> ! {
    if cfg!(debug_assertions) {
        panic!(
            "Mismatched entity and client state: {:?}, {:?}",
            entity, client_state
        )
    } else {
        unsafe { unreachable_unchecked() };
    }
}

pub fn render(entity: &Entity, client_state: &ClientState, pos: &Isometry2<f32>, cur_tick: usize) {
    match (entity, client_state) {
        (Entity::Asteroid(AsteroidEntity { vertices }), ClientState::Asteroid { color }) => {
            let transformed = transform_points(&vertices, pos);
            fill_poly(color, &transformed);
        }
        (Entity::Player(ref player), ClientState::Player { color, .. }) => {
            render_player(player, &pos, color, cur_tick)
        }
        _ => unmatched_state(entity, client_state),
    }
}

fn render_player(player: &PlayerEntity, pos: &Isometry2<f32>, color: &Color, cur_tick: usize) {
    let PlayerEntity {
        size,
        beam_aim,
        beam_on,
        movement: _,
    } = player;
    let transformed = transform_points(&player_verts(*size as f32), pos);
    fill_poly(color, &transformed);

    let beam_gun_len: f32 = 25.;
    let beam_rotation = (pos.translation.vector - Vector2::new(beam_aim.x, beam_aim.y)).normalize();
    let beam_vec = beam_rotation * beam_gun_len;
    let beam_gun_start_point = Point2::new(pos.translation.vector.x, pos.translation.vector.y);
    let beam_gun_endpoint = beam_gun_start_point + beam_vec;

    // Draw beam gun
    render_line(color, 8, beam_gun_start_point, beam_gun_endpoint);

    // Draw beam if beam is active
    if !beam_on {
        return;
    }

    let beam_start = beam_gun_endpoint;
    let beam_endpoint = beam_start + (beam_vec * 10.);

    let ray = Ray::new(beam_gun_endpoint, beam_rotation);
    let mut possible_collisions = Vec::new();
    get_state()
        .world
        .world
        .collision_world()
        .broad_phase()
        .interferences_with_ray(&ray, &mut possible_collisions);
    let broad_phase_miss = possible_collisions.is_empty();

    let collision_check_opt = possible_collisions
        .into_iter()
        .map(|collider_handle| -> Option<(Point2<f32>, f32)> {
            let uuid = get_state().world.handle_map.get(collider_handle).unwrap();
            let EntityHandles {
                entity,
                data: client_state,
                ..
            } = get_state()
                .world
                .uuid_map
                .get(uuid)
                .expect("UUID in `handle_map` but not `uuid_map`");
            let target_pos = get_state()
                .world
                .world
                .collider(*collider_handle)
                .unwrap()
                .position();

            let verts = get_vertices(entity, client_state);
            ray_collision(beam_start, beam_rotation, verts, target_pos)
        }).fold(None, |acc, distance_opt| -> Option<(Point2<f32>, f32)> {
            match (acc, distance_opt) {
                (None, Some(res)) => Some(res),
                (Some((nearest_collision, smallest_distance)), Some((collision, dist))) => {
                    if smallest_distance < dist {
                        Some((nearest_collision, smallest_distance))
                    } else {
                        Some((collision, dist))
                    }
                }
                (Some(acc), None) => Some(acc),
                (None, None) => None,
            }
        });

    let (line_color, beam_endpoint) = if let Some((nearest_collision, _)) = collision_check_opt {
        let drilling_effect = DrillingParticles::new(
            nearest_collision,
            cur_tick,
            5,
            4,
            1.45,
            Color {
                red: 240,
                green: 30,
                blue: 41,
            },
        );
        get_effects_manager().add_effect(box drilling_effect);
        let color = &Color {
            red: 255,
            green: 0,
            blue: 0,
        };
        (color, nearest_collision)
    } else {
        let color = if broad_phase_miss {
            &Color {
                red: 0,
                green: 0,
                blue: 255,
            }
        } else {
            &color
        };
        (color, beam_endpoint)
    };

    render_line(&line_color, 1, beam_start, beam_endpoint);
}

fn get_vertices<'a>(entity: &'a Entity, client_state: &'a ClientState) -> &'a [Point2<f32>] {
    match (entity, client_state) {
        (Entity::Asteroid(AsteroidEntity { vertices, .. }), _) => vertices,
        (Entity::Player(_), ClientState::Player { vertices, .. }) => vertices,
        _ => unmatched_state(entity, client_state),
    }
}

pub fn parse_proto_entity(creation_evt: &CreationEvent) -> Option<EntitySpawn<ClientState>> {
    let (pos, velocity): (Isometry2<f32>, Velocity2<f32>) = creation_evt.get_movement().into();

    let entity: &ProtoEntity = match creation_evt.entity.as_ref() {
        Some(entity) => entity,
        None => {
            error("Received `CreationEvent` without an `enity` field");
            return None;
        }
    };

    let (entity, client_state) = match entity {
        ProtoEntity::player(proto_player) => {
            let size = proto_player.get_size();
            let half_size = (size as f32) / 2.;
            let entity = Entity::Player(PlayerEntity::new(size));
            let client_state = ClientState::Player {
                color: Color::random(),
                vertices: vec![
                    Point2::new(half_size, half_size),
                    Point2::new(-half_size, half_size),
                    Point2::new(-half_size, -half_size),
                    Point2::new(half_size, -half_size),
                ],
            };
            (entity, client_state)
        }
        ProtoEntity::asteroid(asteroid) => {
            let vertices = asteroid
                .get_vert_coords()
                .chunks(2)
                .map(|pts| Point2::new(pts[0], pts[1]))
                .collect();
            let entity = Entity::Asteroid(AsteroidEntity { vertices });
            let client_state = ClientState::Asteroid {
                color: Color::random(),
            };
            (entity, client_state)
        }
    };

    Some(EntitySpawn {
        entity,
        isometry: pos,
        velocity,
        data: client_state,
    })
}
