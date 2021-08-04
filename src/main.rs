#![warn(clippy::all, rust_2018_idioms)]

use std::time::Instant;

use cybersub::{CyberSubApp, MutableResources, ResourcesBuilder};
use macroquad::prelude::{
    clear_background, get_fps, get_frame_time, get_time, load_file, next_frame, Conf, BLACK,
};

fn window_conf() -> Conf {
    Conf {
        window_title: "CyberSub".to_owned(),
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut cybersub_app = CyberSubApp::default();

    if let Ok(grid_file) = load_file("grid.png").await {
        cybersub_app.load_submarine(&grid_file);
    }

    if let Ok(world_file) = load_file("world.png").await {
        cybersub_app.load_rocks(&world_file);
    }

    let background_bytes = load_file("background.png")
        .await
        .expect("Background not found");

    let resources = ResourcesBuilder::new()
        .sub_background(&background_bytes)
        .build();

    let mut mutable_resources = MutableResources::new();

    let mut last_time = None;
    let mut delta_time = || {
        if cfg!(target_arch = "wasm32") {
            0
        } else {
            let last_time = last_time.get_or_insert_with(|| Instant::now());

            let new_time = Instant::now();
            let delta = new_time.saturating_duration_since(*last_time).as_micros() as u32;
            *last_time = new_time;
            delta
        }
    };

    loop {
        clear_background(BLACK);

        egui_macroquad::ui(|egui_ctx| {
            delta_time();

            cybersub_app.draw_ui(egui_ctx);
            cybersub_app.timings.egui_layout = delta_time();

            if !egui_ctx.wants_pointer_input() {
                cybersub_app.handle_pointer_input();
            }
            if !egui_ctx.wants_keyboard_input() {
                cybersub_app.handle_keyboard_input();
            }
            cybersub_app.timings.input_handling = delta_time();
        });

        delta_time();
        cybersub_app.update_game(get_time());
        cybersub_app.timings.game_update = delta_time();

        cybersub_app.draw_game(&resources, &mut mutable_resources);
        cybersub_app.timings.game_layout = delta_time();

        egui_macroquad::draw();
        cybersub_app.timings.egui_drawing = delta_time();

        if cybersub_app.should_quit() {
            return;
        }

        next_frame().await;

        cybersub_app.timings.frame_update = delta_time();
        cybersub_app.timings.fps = get_fps() as u32;
        cybersub_app.timings.frame_time = (get_frame_time() * 1_000_000.0) as u32;
    }
}
