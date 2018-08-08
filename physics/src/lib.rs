#![feature(plugin)]
// #![plugin(rustler_codegen)]

#[macro_use]
extern crate rustler;
#[macro_use]
extern crate lazy_static;
extern crate nphysics2d;
#[macro_use]
extern crate rustler_codegen;
extern crate nalgebra;
extern crate ncollide2d;
extern crate rand;
extern crate uuid;

use rustler::schedule::SchedulerFlags;
use rustler::types::{atom::Atom, ListIterator};
use rustler::{Encoder, Env, NifResult, Term};

pub mod conf;
pub mod physics;
pub mod worldgen;

use self::physics::InternalUserDiff;

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

rustler_export_nifs!(
    "Elixir.NativePhysics",
    [
        ("spawn_user", 1, spawn_user),
        ("tick", 2, tick, SchedulerFlags::DirtyCpu),
        ("get_snapshot", 0, physics::get_snapshot)
    ],
    None
);

#[derive(NifStruct)]
#[module = "NativePhysics.UserDiff"]
pub struct UserDiff<'a> {
    id: String,
    action_type: Atom,
    payload: Term<'a>,
}

fn tick<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
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

    let actions = physics::tick(env, update_all, diffs);

    Ok(actions.encode(env))
}

fn spawn_user<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let uuid = args[0].decode()?;

    let position = physics::spawn_user(uuid);
    Ok(position.encode(env))
}
