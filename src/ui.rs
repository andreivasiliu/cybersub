use egui::{vec2, Color32, Label, Slider};

use crate::{
    app::{GameSettings, GameState, Tool},
    draw::DrawSettings,
    saveload::{load, load_png, save, save_png},
    Timings,
};

pub(crate) struct UiState {
    label: String,
    show_total_water: bool,
    show_ui: bool,
    show_help: bool,
    show_timings: bool,
    show_draw_settings: bool,
    error_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            label: "Hello me!".to_owned(),
            show_total_water: false,
            show_ui: true,
            show_help: false,
            show_timings: false,
            show_draw_settings: false,
            error_message: None,
        }
    }
}

/// Called each time the UI needs repainting, which may be many times per second.
/// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
pub(crate) fn draw_ui(
    ctx: &egui::CtxRef,
    ui_state: &mut UiState,
    settings: &mut GameSettings,
    state: &mut GameState,
    draw_settings: &mut DrawSettings,
    timings: &Timings,
) {
    let UiState {
        label,
        show_total_water,
        show_ui,
        show_help,
        show_timings,
        show_draw_settings,
        error_message,
    } = ui_state;

    let GameSettings {
        enable_gravity,
        enable_inertia,
        camera,
        current_tool,
        quit_game,
        ..
    } = settings;

    let GameState {
        water_grid: grid, ..
    } = state;

    let DrawSettings {
        draw_sea,
        draw_rocks,
        draw_objects,
        draw_walls,
        draw_wires,
        draw_water,
    } = draw_settings;

    if *show_ui {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Save grid").clicked() {
                        match save(grid) {
                            Ok(()) => (),
                            Err(err) => *error_message = Some(err),
                        }
                    }

                    if ui.button("Load grid").clicked() {
                        match load() {
                            Ok(new_grid) => *grid = new_grid,
                            Err(err) => *error_message = Some(err),
                        }
                    }

                    if ui.button("Save grid as PNG").clicked() {
                        match save_png(grid) {
                            Ok(()) => (),
                            Err(err) => *error_message = Some(err),
                        }
                    }

                    if ui.button("Load grid from PNG").clicked() {
                        match load_png() {
                            Ok(new_grid) => *grid = new_grid,
                            Err(err) => *error_message = Some(err),
                        }
                    }

                    ui.separator();
                    if ui.button("Show total water").clicked() {
                        *show_total_water = !*show_total_water;
                    }
                    if ui.button("Toggle gravity").clicked() {
                        *enable_gravity = !*enable_gravity;
                    }
                    if ui.button("Toggle inertia").clicked() {
                        *enable_inertia = !*enable_inertia;
                    }
                    if cfg!(not(target_arch = "wasm32")) && ui.button("Show timings").clicked() {
                        *show_timings = !*show_timings;
                    }
                    if ui.button("Clear water").clicked() {
                        grid.clear();
                    }
                    if ui.button("Help").clicked() {
                        *show_help = true;
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        *quit_game = true;
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Powered by");
                ui.add(egui::Hyperlink::new("https://github.com/emilk/egui/").text("egui"));
                ui.label("and");
                ui.add(
                    egui::Hyperlink::new("https://github.com/not-fl3/macroquad/").text("macroquad"),
                );
                egui::warn_if_debug_build(ui);
                ui.label(format!("FPS:"));
                ui.colored_label(Color32::GREEN, timings.fps.to_string());
                ui.label(format!("x:"));
                ui.colored_label(Color32::GREEN, camera.pointing_at.0.to_string());
                ui.label(format!("y:"));
                ui.colored_label(Color32::GREEN, camera.pointing_at.1.to_string());

                if *show_total_water {
                    ui.label(format!("Total water: {}", grid.total_water()));
                }
            });
        });
    }

    egui::Window::new("egui ❤ macroquad").show(ctx, |ui| {
        ui.checkbox(show_ui, "Show UI");
        ui.checkbox(enable_gravity, "Enable gravity");
        ui.checkbox(enable_inertia, "Enable inertia");
        ui.checkbox(show_draw_settings, "Show draw settings");
        if cfg!(not(target_arch = "wasm32")) {
            // Timing not available in browsers
            ui.checkbox(show_timings, "Show timings");
        }
        ui.horizontal(|ui| {
            ui.label("Zoom:");
            ui.add(Slider::new(&mut camera.zoom, -512..=36));
        });
    });

    let toolbar = egui::Window::new("toolbar")
        .auto_sized()
        .title_bar(false)
        .default_pos(ctx.available_rect().left_bottom() + vec2(16.0, -16.0 - 32.0));

    toolbar.show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.radio_value(current_tool, Tool::AddWater, "Add Water");
            ui.radio_value(current_tool, Tool::AddWall, "Add Walls");
            ui.radio_value(current_tool, Tool::AddOrangeWire, "Orange Wires");
            ui.radio_value(current_tool, Tool::AddBrownWire, "Brown Wires");
            ui.radio_value(current_tool, Tool::AddBlueWire, "Blue Wires");
            ui.radio_value(current_tool, Tool::AddGreenWire, "Green Wires");
            ui.radio_value(current_tool, Tool::RemoveWall, "Remove Walls");
        });
    });

    if *show_draw_settings {
        egui::Window::new("Draw settings").show(ctx, |ui| {
            ui.checkbox(draw_sea, "Enable sea shader");
            ui.checkbox(draw_rocks, "Draw rocks");
            ui.checkbox(draw_objects, "Draw objects");
            ui.checkbox(draw_walls, "Draw walls");
            ui.checkbox(draw_wires, "Draw wires");
            ui.checkbox(draw_water, "Draw water");

            if ui.button("Close").clicked() {
                *show_draw_settings = false;
            }
        });
    }

    if *show_timings {
        egui::Window::new("Timings").show(ctx, |ui| {
            let mut show_timer = |name: &str, value: u32| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", name));
                    ui.add(
                        Label::new(format!("{:5}", value))
                            .text_color(Color32::GREEN)
                            .monospace(),
                    )
                });
            };

            show_timer("egui_layout", timings.egui_layout);
            show_timer("egui_drawing", timings.egui_drawing);
            show_timer("input_handling", timings.input_handling);
            show_timer("game_update", timings.game_update);
            show_timer("game_layout", timings.game_layout);
            show_timer("frame_update", timings.frame_update);

            if ui.button("Close").clicked() {
                *show_timings = false;
            }
        });
    }

    if error_message.is_some() {
        egui::Window::new("Error").show(ctx, |ui| {
            ui.label(error_message.as_ref().unwrap());

            if ui.button("Close").clicked() {
                *error_message = None;
            }
        });
    }

    if *show_help {
        egui::Window::new("Cybersub prototype").show(ctx, |ui| {
            ui.label("This is a water simulation prototype that I was considering using for a little game.");
            ui.label("The code is here:");
            ui.hyperlink_to("https://github.com/andreivasiliu/cybersub", "https://github.com/andreivasiliu/cybersub");
            ui.label("Left-click to add water or interact with objects.");
            ui.label("On browsers, the right-click menu is disabled, in order to make scrolling easier. You can still shift-right-click.");
            ui.label("WASD, arrow keys, or hold right-click to move camera. Keypad +/- or mouse-scroll to zoom. Minus doesn't work on browsers. No idea why. There is currently no way to move the camera with a touch-screen.");
            ui.label("Use the tool controls (Add Water, Add Walls, etc) at the bottom to switch what left-click paints. Holding shift or ctrl is a shortcut for switching.");
            ui.label("Firefox is having issues with rendering the whole thing; my phone and other browsers work fine though.");

            if ui.button("Close").clicked() {
                *show_help = false;
            }
        });
    }
}
