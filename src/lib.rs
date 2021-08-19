#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod collisions;
mod draw;
mod input;
mod objects;
mod resources;
mod rocks;
mod saveload;
mod sonar;
mod ui;
mod update;
mod water;
mod wires;

pub use app::{CyberSubApp, Timings};
pub use saveload::SubmarineFileData;
