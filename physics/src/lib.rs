#![feature(extern_prelude, nll)]

#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate ncollide2d;
extern crate nphysics2d;
extern crate rand;
extern crate uuid;

#[cfg(feature = "elixir-interop")]
#[cfg_attr(feature = "elixir-interop", macro_use)]
extern crate rustler;

#[cfg(feature = "elixir-interop")]
#[cfg_attr(feature = "elixir-interop", macro_use)]
extern crate rustler_codegen;

pub mod conf;
pub mod physics;
pub mod worldgen;

#[cfg(feature = "elixir-interop")]
pub mod atoms {
    rustler_atoms! {
        // Movement Directions
        atom UP;
        atom UP_RIGHT;
        atom RIGHT;
        atom DOWN_RIGHT;
        atom DOWN;
        atom DOWN_LEFT;
        atom LEFT;
        atom UP_LEFT;
        atom STOP;

        // Action Types
        atom direction;
        atom beam_rotation;
        atom beam_toggle;

        // Update Types
        atom isometry;
        atom beam_event;
        atom username;

        // Entity Types
        atom player;
        atom asteroid;
        atom barrier;

        // Proximity Events
        atom intersecting;
        atom disjoint;

        // Map/Struct Keys
        atom x;
        atom y;
        atom size;
        atom movement;
        atom beam_aim;
        atom beam_on;
        atom vert_coords;
    }
}

#[cfg(feature = "elixir-interop")]
pub mod ext {
    use rustler::error::Error as NifError;
    use rustler::schedule::SchedulerFlags;
    use rustler::types::{atom::Atom, ListIterator};
    use rustler::{Encoder, Env, NifResult, Term};

    use super::atoms;
    use super::physics::{
        server::{InternalUserDiff, InternalUserDiffAction},
        Movement,
    };

    rustler_export_nifs!(
        "Elixir.NativePhysics",
        [
            ("spawn_user", 1, spawn_user),
            ("tick", 2, tick, SchedulerFlags::DirtyCpu),
            ("get_snapshot", 0, super::physics::server::get_snapshot)
        ],
        None
    );

    #[derive(NifStruct)]
    #[module = "NativePhysics.UserDiff"]
    pub struct UserDiff<'a> {
        pub id: String,
        pub action_type: Atom,
        pub payload: Term<'a>,
    }

    impl<'a> UserDiff<'a> {
        pub fn parse(self, env: Env<'a>) -> NifResult<InternalUserDiff> {
            let internal_action = match self.action_type {
                t if atoms::direction() == t => {
                    let movement = Movement::from_term(self.payload)?;
                    InternalUserDiffAction::Movement(movement)
                }
                t if atoms::beam_rotation() == t => {
                    let x: u32 = self.payload.map_get(atoms::x().encode(env))?.decode()?;
                    let y: u32 = self.payload.map_get(atoms::y().encode(env))?.decode()?;
                    InternalUserDiffAction::BeamAim {
                        x: x as f32,
                        y: y as f32,
                    }
                }
                t if atoms::beam_toggle() == t => {
                    let beam_on: bool = self.payload.decode()?;
                    InternalUserDiffAction::BeamToggle(beam_on)
                }
                t if atoms::username() == t => {
                    let username: String = self.payload.decode()?;
                    InternalUserDiffAction::Username(username)
                }
                _ => Err(NifError::Atom("invalid_action_type"))?,
            };

            Ok(InternalUserDiff {
                id: self.id,
                action: internal_action,
            })
        }
    }

    pub fn tick<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
        let diffs_iterator: ListIterator = args[0].decode()?;
        let update_all: bool = args[1].decode()?;

        let diffs: Vec<InternalUserDiff> = diffs_iterator
            .map(|diff| -> NifResult<UserDiff<'a>> { diff.decode() })
            .map(
                |diff_res: NifResult<UserDiff>| -> NifResult<InternalUserDiff> {
                    match diff_res {
                        Ok(diff) => diff.parse(env),
                        Err(err) => Err(err),
                    }
                },
            ).collect::<NifResult<Vec<InternalUserDiff>>>()?;

        let actions = super::physics::server::tick(env, update_all, diffs);

        Ok(actions.encode(env))
    }

    pub fn spawn_user<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
        let uuid = args[0].decode()?;

        let position = super::physics::server::spawn_user(uuid);
        Ok(position.encode(env))
    }
}
