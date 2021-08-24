#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod client;
mod draw;
mod game_state;
mod input;
mod resources;
mod saveload;
#[cfg(not(target_arch = "wasm32"))]
mod server;
mod ui;

pub use app::{CyberSubApp, Timings};
pub use saveload::SubmarineFileData;
