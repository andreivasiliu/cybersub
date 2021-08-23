#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod draw;
mod game_state;
mod input;
mod resources;
mod saveload;
mod server;
mod ui;

pub use app::{CyberSubApp, Timings};
pub use saveload::SubmarineFileData;
