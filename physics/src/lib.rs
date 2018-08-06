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

pub mod physics;
pub mod worldgen;

pub mod atoms {
    rustler_atoms! {
        atom UP;
        atom UP_RIGHT;
        atom RIGHT;
        atom DOWN_RIGHT;
        atom DOWN;
        atom DOWN_LEFT;
        atom LEFT;
        atom UP_LEFT;
        atom STOP;
    }
}

// use rustler::{NifDecoder, NifEncoder, NifEnv, NifError, NifTerm};

// rustler_export_nifs!("Elixir.NativePhysics", []);
