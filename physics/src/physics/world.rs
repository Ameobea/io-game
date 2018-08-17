use conf::CONF;

use std::collections::BTreeMap;
use std::usize;

use nalgebra::{Isometry2, Vector2};
use nphysics2d::algebra::Velocity2;
use nphysics2d::force_generator::{ForceGenerator, ForceGeneratorHandle};
use nphysics2d::object::{BodyHandle, BodySet, BodyStatus, ColliderHandle, Material, RigidBody};
use nphysics2d::solver::{IntegrationParameters, SignoriniModel};
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World;
use uuid::Uuid;

use super::entities::{Entity, EntityHandles, EntitySpawn, PlayerEntity};
use super::Movement;
use worldgen::get_initial_entities;

pub const COLLIDER_MARGIN: f32 = CONF.physics.collider_margin;
const WORLD_MISSING_ERR: &'static str = "Entity in UUID map but not the world!";

#[cfg(feature = "elixir-interop")]
mod cond {
    use uuid::Uuid;

    pub type EntityKey = String;

    #[inline(always)]
    pub fn uuid_to_key(id: Uuid) -> EntityKey {
        id.to_string()
    }
}

#[cfg(not(feature = "elixir-interop"))]
mod cond {
    use uuid::Uuid;

    pub type EntityKey = Uuid;

    #[inline(always)]
    pub fn uuid_to_key(id: Uuid) -> EntityKey {
        id
    }
}

use self::cond::*;

pub struct PhysicsWorldInner<T = ()> {
    /// Maps UUIDs to internal physics entity handles
    pub uuid_map: BTreeMap<EntityKey, EntityHandles<T>>,
    /// Maps `ColliderHandle`s to UUIDs
    pub handle_map: BTreeMap<ColliderHandle, EntityKey>,
    /// The inner physics world that contains all of the entities' geometry and physics data
    pub world: World<f32>,
    /// A list containing handles to all player entities, used to apply movement and friction
    pub user_handles: Vec<(BodyHandle, EntityKey, ForceGeneratorHandle)>,
    /// Maps the collider handles of beam sensors to the User entities that own them
    pub beam_sensors: BTreeMap<ColliderHandle, EntityKey>,
}

impl PhysicsWorldInner<()> {
    pub fn initialize(&mut self) {
        // Populate the world with initial entities
        for entity_spawn in get_initial_entities() {
            self.spawn_entity(Uuid::new_v4(), entity_spawn);
        }
    }
}

pub struct PlayerMovementForceGenerator {
    movement: Movement,
    player_body_handle: BodyHandle,
}

impl PlayerMovementForceGenerator {
    pub fn new(player_body_handle: BodyHandle, movement: Movement) -> Self {
        PlayerMovementForceGenerator {
            movement,
            player_body_handle,
        }
    }
}

impl ForceGenerator<f32> for PlayerMovementForceGenerator {
    fn apply(&mut self, _: &IntegrationParameters<f32>, bodies: &mut BodySet<f32>) -> bool {
        let mut acceleration: Vector2<f32> = self.movement.into();
        acceleration *= CONF.physics.acceleration_per_tick;
        let acceleration = Velocity2::new(acceleration, 0.0);
        let mut part = bodies.body_part_mut(self.player_body_handle);
        let force = part.as_ref().inertia() * acceleration;
        part.apply_force(&force);

        true
    }
}

impl<T> PhysicsWorldInner<T> {
    pub fn new() -> Self {
        let mut world = World::new();
        // world.set_contact_model(SignoriniModel::new());
        world.set_timestep(CONF.physics.engine_time_step);

        PhysicsWorldInner {
            uuid_map: BTreeMap::new(),
            handle_map: BTreeMap::new(),
            world,
            user_handles: Vec::new(),
            beam_sensors: BTreeMap::new(),
        }
    }

    /// Apply movement updates to all user entities based on their input and apply friction.  Then,
    /// step the underlying physics world for one tick of the simulation.
    pub fn step(&mut self) {
        for (user_body_handle, uuid, force_gen_handle) in &mut self.user_handles {
            let EntityHandles { entity, .. } = self
                .uuid_map
                .get(uuid)
                .expect("UUID in `user_handles` not in `uuid_map`");
            let movement: Movement = match entity {
                Entity::Player(PlayerEntity { movement, .. }) => (*movement),
                _ => panic!("Expected a player entity but the entity data wasn't one!"),
            };

            let user_rigid_body: &mut RigidBody<f32> = self
                .world
                .rigid_body_mut(*user_body_handle)
                .expect("ERROR: Player wasn't a rigid body!");

            // The physics engine puts entities to sleep if their energies are low enough, causing
            // them to not be simulated.  We manually wake up the player to ensure that the changes
            // we apply to their velocities from movement directions are taken into account by the
            // physics engine.
            user_rigid_body.activate();

            self.world.remove_force_generator(*force_gen_handle);
            let new_force_gen = PlayerMovementForceGenerator::new(*user_body_handle, movement);
            let new_force_gen_handle = self.world.add_force_generator(new_force_gen);
            *force_gen_handle = new_force_gen_handle;

            // user_rigid_body.set_velocity(friction_adjusted_new_velocity);
        }

        // Step the physics simulation
        self.world.step();
    }

    pub fn spawn_entity(&mut self, uuid: Uuid, entity_data: EntitySpawn<T>) {
        let EntitySpawn {
            entity,
            isometry,
            velocity,
            data,
            body_status,
        } = entity_data;

        let shape_handle = entity.get_shape_handle();
        let inertia = shape_handle.inertia(entity.get_density());
        let center_of_mass = shape_handle.center_of_mass();

        let (collider_handle, body_handle) = if body_status == BodyStatus::Static {
            let collider_handle = self.world.add_collider(
                COLLIDER_MARGIN,
                shape_handle,
                BodyHandle::ground(),
                isometry,
                Material::default(),
            );

            (collider_handle, BodyHandle::ground())
        } else {
            let body_handle = self.world.add_rigid_body(isometry, inertia, center_of_mass);

            let body = self.world.rigid_body_mut(body_handle).unwrap();
            body.set_velocity(velocity);
            body.set_status(body_status);

            let collider_handle = self.world.add_collider(
                COLLIDER_MARGIN,
                shape_handle,
                body_handle,
                Isometry2::identity(),
                Material::default(),
            );

            if let Entity::Player(_) = entity {
                let force_generator =
                    PlayerMovementForceGenerator::new(body_handle, Movement::Stop);
                let force_gen_handle = self.world.add_force_generator(force_generator);
                self.user_handles
                    .push((body_handle, uuid_to_key(uuid), force_gen_handle));
            }

            (collider_handle, body_handle)
        };
        let handles = EntityHandles {
            collider_handle,
            body_handle,
            beam_handle: None,
            entity,
            data,
        };
        self.uuid_map.insert(uuid_to_key(uuid), handles);
        self.handle_map.insert(collider_handle, uuid_to_key(uuid));
    }

    /// Removes an entity from both the physics world as well as all maps.
    pub fn remove_entity(&mut self, entity_id: &EntityKey) {
        let EntityHandles {
            collider_handle,
            body_handle,
            beam_handle,
            ..
        } = match self.uuid_map.remove(entity_id) {
            Some(handles) => handles,
            None => {
                println!("ERROR: Tried to remove entity but it didn't exist in the UUID map");
                return;
            }
        };

        self.handle_map.remove(&collider_handle);
        self.world.remove_colliders(&[collider_handle]);
        self.world.remove_bodies(&[body_handle]);
        // TODO: Possibly convert this to a map
        let mut pos = usize::max_value();
        for (i, (candidate_body_handle, uuid, _)) in self.user_handles.iter().enumerate() {
            if (candidate_body_handle, uuid) == (&body_handle, entity_id) {
                pos = i;
                break;
            }
        }
        if pos != usize::max_value() {
            self.user_handles.swap_remove(pos);
        }

        if let Some(beam_handle) = beam_handle {
            self.world.remove_colliders(&[beam_handle]);
            self.beam_sensors.remove(&beam_handle);
        }
    }

    /// Sets the movement input for a player
    pub fn set_player_movement(&mut self, user_id: &EntityKey, new_movement: Movement) {
        let EntityHandles { entity, .. } = match self.uuid_map.get_mut(user_id) {
            Some(handles) => handles,
            None => {
                println!(
                    "Attempted to set movement of entity id {} but it's not in the UUID map",
                    user_id
                );
                return;
            }
        };

        match entity {
            Entity::Player(PlayerEntity {
                ref mut movement, ..
            }) => *movement = new_movement,
            _ => panic!("Tried to set movement for non-player entity: {:?}", entity),
        }
    }

    /// Updates the position, movement, and physics dynamics for an entity in the world
    pub fn update_movement(
        &mut self,
        entity_id: &EntityKey,
        pos: &Isometry2<f32>,
        velocity: &Velocity2<f32>,
    ) {
        let EntityHandles {
            body_handle,
            collider_handle,
            ..
        } = match self.uuid_map.get(entity_id) {
            Some(handles) => handles,
            None => {
                println!(
                    "ERROR: Tried to update entity movement but it didn't exist in the UUID map"
                );
                return;
            }
        };

        // Set the velocity of the `RigidBody`
        let rigid_body = self
            .world
            .rigid_body_mut(*body_handle)
            .expect(WORLD_MISSING_ERR);
        rigid_body.set_velocity(*velocity);

        let collider = self
            .world
            .collision_world_mut()
            .collision_object_mut(*collider_handle)
            .expect("ERROR: `collider_handle` in `uuid_map` but not the world");
        let pos_wrt_body = *collider.data().position_wrt_body();

        // Set the position of the attached `CollisionObject`
        collider.set_position(*pos);
        self.world
            .collision_world_mut()
            .set_position(*collider_handle, pos_wrt_body * *pos);
    }

    /// Removes all entities from this world
    pub fn clear(&mut self) {
        for (
            _,
            EntityHandles {
                collider_handle,
                body_handle,
                beam_handle,
                ..
            },
        ) in self.uuid_map.iter()
        {
            self.world.remove_colliders(&[*collider_handle]);
            self.world.remove_bodies(&[*body_handle]);
            if let Some(beam_handle) = beam_handle {
                self.world.remove_colliders(&[*beam_handle]);
            }
        }

        self.uuid_map.clear();
        self.handle_map.clear();
        self.user_handles.clear();
        self.beam_sensors.clear();
    }
}
