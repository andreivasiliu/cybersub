#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod draw;
mod input;
mod objects;
mod saveload;
mod ui;
mod water;
mod wires;

pub use app::{CyberSubApp, Timings};
pub use draw::{Resources, ResourcesBuilder};
