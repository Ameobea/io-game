use conf::CONF;

use std::collections::BTreeMap;

use nalgebra::{Isometry2, Vector2};
use nphysics2d::algebra::Velocity2;
use nphysics2d::object::{BodyHandle, ColliderHandle, Material, RigidBody, SensorHandle};
use nphysics2d::solver::SignoriniModel;
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World;
use uuid::Uuid;

use super::entities::EntityType;
use super::Movement;
use worldgen::{get_initial_entities, EntitySpawn};

pub const COLLIDER_MARGIN: f32 = CONF.physics.collider_margin;
pub const DEFAULT_PLAYER_SIZE: f32 = CONF.game.default_player_size;

pub struct EntityHandles {
    pub collider_handle: ColliderHandle,
    pub body_handle: BodyHandle,
    pub beam_handle: Option<SensorHandle>,
}

pub struct PhysicsWorldInner {
    /// Maps UUIDs to internal physics entity handles
    pub uuid_map: BTreeMap<String, EntityHandles>,
    /// Maps `ColliderHandle`s to UUIDs
    pub handle_map: BTreeMap<ColliderHandle, (String, EntityType)>,
    pub world: World<f32>,
    pub user_handles: Vec<(BodyHandle, ColliderHandle)>,
    /// Maps the collider handles of beam sensors to the User entities that own them
    pub beam_sensors: BTreeMap<ColliderHandle, String>,
}

impl PhysicsWorldInner {
    pub fn new() -> Self {
        let mut world = World::new();
        world.set_contact_model(SignoriniModel::new());
        world.set_timestep(CONF.physics.engine_time_step);

        let mut uuid_map = BTreeMap::new();
        let mut handle_map = BTreeMap::new();

        // Populate the world with initial entities
        for EntitySpawn {
            isometry,
            velocity,
            entity,
        } in get_initial_entities()
        {
            let shape_handle = entity.get_shape_handle();
            let inertia = shape_handle.inertia(entity.get_density());
            let center_of_mass = shape_handle.center_of_mass();
            let body_handle = world.add_rigid_body(isometry, inertia, center_of_mass);
            {
                world
                    .rigid_body_mut(body_handle)
                    .unwrap()
                    .set_velocity(velocity);
            }

            let collider_handle = world.add_collider(
                COLLIDER_MARGIN,
                shape_handle,
                body_handle,
                Isometry2::identity(),
                Material::default(),
            );

            let uuid = Uuid::new_v4();
            let handles = EntityHandles {
                collider_handle,
                body_handle,
                beam_handle: None,
            };
            uuid_map.insert(uuid.to_string(), handles);
            handle_map.insert(collider_handle, (uuid.to_string(), entity));
        }

        PhysicsWorldInner {
            uuid_map,
            handle_map,
            world,
            user_handles: Vec::new(),
            beam_sensors: BTreeMap::new(),
        }
    }

    /// Apply movement updates to all user entities based on their input and apply friction.  Then,
    /// step the underlying physics world for one tick of the simulation.
    pub fn step(&mut self) {
        for (user_body_handle, user_collider_handle) in &self.user_handles {
            let user_rigid_body: &mut RigidBody<f32> = self
                .world
                .rigid_body_mut(*user_body_handle)
                .expect("ERROR: Player wasn't a rigid body!");

            let (_uuid, user_data) = self
                .handle_map
                .get(user_collider_handle)
                .expect("User collider handle isn't in the `handle_map`!");
            let movement: Movement = match user_data {
                EntityType::Player { movement, .. } => (*movement),
                _ => panic!("Expected a player entity but the entity data wasn't one!"),
            };

            // The physics engine puts entities to sleep if their energies are low enough, causing
            // them to not be simulated.  We manually wake up the player to ensure that the changes
            // we apply to their velocities from movement directions are taken into account by the
            // physics engine.unreachable!
            user_rigid_body.activate();

            // Apply thrust force from movement input
            let velocity = *user_rigid_body.velocity();
            let mut movement_force: Vector2<f32> = movement.into();
            movement_force *= CONF.physics.acceleration_per_tick;
            let mut new_velocity =
                Velocity2::new(velocity.linear + movement_force, velocity.angular);

            // Apply friction
            let friction_adjusted_new_velocity = new_velocity;

            user_rigid_body.set_velocity(friction_adjusted_new_velocity);
        }

        // Step the physics simulation
        self.world.step();
    }
}
