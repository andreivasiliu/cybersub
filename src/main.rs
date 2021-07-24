#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use cybersub::CyberSubApp;
use macroquad::prelude::{clear_background, get_time, next_frame, Conf, BLACK};

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

    loop {
        clear_background(BLACK);

        egui_macroquad::ui(|egui_ctx| {
            cybersub_app.draw_ui(egui_ctx);
            if !egui_ctx.wants_pointer_input() {
                cybersub_app.handle_pointer_input();
            }
            if !egui_ctx.wants_keyboard_input() {
                cybersub_app.handle_keyboard_input();
            }
        });

        cybersub_app.update_game(get_time());

        cybersub_app.draw_game();

        egui_macroquad::draw();

        if cybersub_app.should_quit() {
            return;
        }

        next_frame().await
    }
}
