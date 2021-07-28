#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::CyberSubApp;

mod draw;
mod input;
mod objects;
mod saveload;
mod ui;
mod water;

pub use draw::{Resources, ResourcesBuilder};
