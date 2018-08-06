#![feature(plugin, try_from)]
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

use std::convert::TryInto;

use rustler::types::ListIterator;
use rustler::{Encoder, Env, NifResult, Term};

pub mod physics;
pub mod worldgen;

use self::physics::{InternalUserDiff, UserDiff};

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
        atom movement;
        atom beam_aim;
        atom beam_toggle;

        // Update Types
        atom isometry;

        // Entity Types
        atom player;
        atom asteroid;
    }
}

rustler_export_nifs!(
    "Elixir.NativePhysics",
    [
        ("spawn_user", 1, spawn_user),
        ("tick", 2, tick),
        ("get_snapshot", 0, get_snapshot)
    ],
    None
);

fn get_snapshot<'a>(env: Env<'a>, _args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let snapshot = physics::get_snapshot(env)?;
    Ok(snapshot.encode(env))
}

fn tick<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let diffs_iterator: ListIterator = args[0].decode()?;
    let update_all: bool = args[1].decode()?;

    let diffs: Vec<InternalUserDiff> = diffs_iterator
        .map(|diff| diff.decode())
        .map(
            |diff_res: NifResult<UserDiff>| -> NifResult<InternalUserDiff> {
                match diff_res {
                    Ok(diff) => diff.try_into(),
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
