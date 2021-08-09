#![warn(clippy::all, rust_2018_idioms)]

use std::{path::Path, time::Instant};

use cybersub::{CyberSubApp, SubmarineFileData};
use macroquad::prelude::{
    clear_background, get_fps, get_frame_time, get_time, load_file, next_frame,
    set_pc_assets_folder, Conf, BLACK,
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
async fn main() -> Result<(), String> {
    let mut cybersub_app = CyberSubApp::default();

    if cfg!(not(target_arch = "wasm32")) {
        // Share the world and submarine assets with the WASM directory for Github Pages
        // Will eventually remove this in favor of building the pages via the actions workflow
        set_pc_assets_folder("docs");
    }

    let world = load_file("world.png")
        .await
        .map_err(|err| err.to_string())?;
    cybersub_app.load_rocks(&world);

    let dugong = load_submarine_files("dugong").await?;
    cybersub_app.load_submarine(dugong)?;

    let mut last_time = None;
    let mut delta_time = || {
        if cfg!(target_arch = "wasm32") {
            0
        } else {
            let last_time = last_time.get_or_insert_with(Instant::now);

            let new_time = Instant::now();
            let delta = new_time.saturating_duration_since(*last_time).as_micros() as u32;
            *last_time = new_time;
            delta
        }
    };

    loop {
        clear_background(BLACK);

        delta_time();
        cybersub_app.update_game(get_time());
        cybersub_app.timings.game_update = delta_time();

        cybersub_app.draw_game();
        cybersub_app.timings.game_layout = delta_time();

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

        egui_macroquad::draw();
        cybersub_app.timings.egui_drawing = delta_time();

        if cybersub_app.should_quit() {
            return Ok(());
        }

        next_frame().await;

        cybersub_app.timings.frame_update = delta_time();
        cybersub_app.timings.fps = get_fps() as u32;
        cybersub_app.timings.frame_time = (get_frame_time() * 1_000_000.0) as u32;
    }
}

async fn load_submarine_files(name: &str) -> Result<SubmarineFileData, String> {
    let path = match Path::new(name).file_name() {
        Some(file_name) => Path::new(file_name),
        None => return Err("Submarine path must be a simple file name".to_string()),
    };

    let load_sub_file = |file_name| async move {
        let sub_path = path.join(file_name);
        load_file(&sub_path.to_string_lossy()).await.map_err(|err| {
            format!(
                "Could not load file {} for submarine {}: {}",
                file_name,
                sub_path.to_string_lossy(),
                err
            )
        })
    };

    let water_grid = load_sub_file("water_grid.png").await?;
    let background = load_sub_file("background.png").await?;
    let objects = load_sub_file("objects.yaml").await?;

    Ok(SubmarineFileData {
        water_grid,
        background,
        objects,
    })
}
