#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::CyberSubApp;

mod draw;
mod draw_quad;
mod input;
mod saveload;
mod water;

pub use draw::{Resources, ResourcesBuilder};
