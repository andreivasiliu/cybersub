use egui::{vec2, Align2, Color32, Label, Slider, Ui};

use crate::{
    app::{GameSettings, GameState, Tool, UpdateSettings},
    draw::DrawSettings,
    objects::compute_navigation,
    saveload::{load, load_png, save, save_png},
    Timings,
};

pub(crate) struct UiState {
    show_total_water: bool,
    show_bars: bool,
    show_main_settings: bool,
    show_toolbar: bool,
    show_help: bool,
    show_timings: bool,
    show_navigation_info: bool,
    show_draw_settings: bool,
    show_update_settings: bool,
    error_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_total_water: false,
            show_bars: true,
            show_main_settings: true,
            show_toolbar: true,
            show_help: false,
            show_timings: false,
            show_navigation_info: false,
            show_draw_settings: false,
            show_update_settings: false,
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
    update_settings: &mut UpdateSettings,
    timings: &Timings,
) {
    let UiState {
        show_total_water,
        show_bars,
        show_toolbar,
        show_main_settings,
        show_help,
        show_timings,
        show_navigation_info,
        show_draw_settings,
        show_update_settings,
        error_message,
    } = ui_state;

    let GameSettings {
        enable_gravity,
        enable_inertia,
        camera,
        current_submarine,
        current_tool,
        quit_game,
        ..
    } = settings;

    let GameState { submarines, .. } = state;

    let DrawSettings {
        draw_egui,
        draw_sea_dust,
        draw_sea_caustics,
        draw_rocks,
        draw_background,
        draw_objects,
        draw_walls,
        draw_wires,
        draw_water,
        draw_sonar,
        draw_engine_turbulence,
    } = draw_settings;

    let UpdateSettings {
        update_water,
        update_wires,
        update_sonar,
        update_objects,
        update_position,
        update_collision,
    } = update_settings;

    if *show_bars {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if let Some(submarine) = submarines.get_mut(*current_submarine) {
                        let grid = &mut submarine.water_grid;

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

                        if ui.button("Clear water").clicked() {
                            grid.clear();
                        }
                    } else {
                        ui.label("<no submarine selected>");
                    }

                    if ui.button("Show total water").clicked() {
                        *show_total_water = !*show_total_water;
                    }
                    ui.separator();

                    if ui.button("Help").clicked() {
                        *show_help = true;
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        *quit_game = true;
                    }
                });
                egui::menu::menu(ui, "View", |ui| {
                    if ui.button("Show toolbar").clicked() {
                        *show_toolbar = !*show_toolbar;
                    }
                    if ui.button("Show main settings").clicked() {
                        *show_main_settings = !*show_main_settings;
                    }
                    if ui.button("Show navigation info").clicked() {
                        *show_navigation_info = !*show_navigation_info;
                    }
                    if ui.button("Show draw settings").clicked() {
                        *show_draw_settings = !*show_draw_settings;
                    }
                    if ui.button("Show update settings").clicked() {
                        *show_update_settings = !*show_update_settings;
                    }
                    if cfg!(not(target_arch = "wasm32")) && ui.button("Show timings").clicked() {
                        *show_timings = !*show_timings;
                    }
                });
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
                ui.label("FPS:".to_string());
                ui.colored_label(Color32::GREEN, timings.fps.to_string());
                ui.label("x:".to_string());
                ui.colored_label(Color32::GREEN, camera.pointing_at.0.to_string());
                ui.label("y:".to_string());
                ui.colored_label(Color32::GREEN, camera.pointing_at.1.to_string());

                if let Some(submarine) = submarines.get_mut(*current_submarine) {
                    ui.label("speed:".to_string());
                    ui.colored_label(Color32::YELLOW, submarine.navigation.speed.0.to_string());
                    ui.label("/".to_string());
                    ui.colored_label(Color32::YELLOW, submarine.navigation.speed.1.to_string());
                    ui.label("acceleration:".to_string());
                    ui.colored_label(
                        Color32::YELLOW,
                        submarine.navigation.acceleration.0.to_string(),
                    );
                    ui.label("/".to_string());
                    ui.colored_label(
                        Color32::YELLOW,
                        submarine.navigation.acceleration.1.to_string(),
                    );
                }

                if *show_total_water {
                    if let Some(submarine) = submarines.get(*current_submarine) {
                        ui.label(format!(
                            "Total water: {}",
                            submarine.water_grid.total_water()
                        ));
                    }
                }
            });
        });
    }

    if *show_main_settings {
        egui::Window::new("Settings").show(ctx, |ui| {
            ui.checkbox(show_toolbar, "Show toolbar");
            ui.checkbox(show_main_settings, "Show main settings");
            ui.checkbox(show_navigation_info, "Show navigation info");
            ui.checkbox(show_draw_settings, "Show draw settings");
            ui.checkbox(show_update_settings, "Show update settings");
            if cfg!(not(target_arch = "wasm32")) {
                // Timing not available in browsers
                ui.checkbox(show_timings, "Show timings");
            }
            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(Slider::new(&mut camera.zoom, -512..=36));
            });
        });
    }

    if *show_toolbar {
        let toolbar = egui::Window::new("toolbar")
            .auto_sized()
            .title_bar(false)
            .anchor(Align2::LEFT_BOTTOM, vec2(16.0, -16.0));

        toolbar.show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.radio_value(current_tool, Tool::AddWater, "Add Water");
                ui.radio_value(current_tool, Tool::AddWall, "Add Walls");
                ui.radio_value(current_tool, Tool::AddPurpleWire, "Purple Wires");
                ui.radio_value(current_tool, Tool::AddBrownWire, "Brown Wires");
                ui.radio_value(current_tool, Tool::AddBlueWire, "Blue Wires");
                ui.radio_value(current_tool, Tool::AddGreenWire, "Green Wires");
                ui.radio_value(current_tool, Tool::RemoveWall, "Remove Walls");
            });
        });
    }

    if *show_navigation_info {
        egui::Window::new("Navigation info").show(ctx, |ui| {
            if let Some(submarine) = submarines.get_mut(*current_submarine) {
                fn add_info(ui: &mut Ui, label: &str, value: (i32, i32)) {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", label));
                        ui.colored_label(Color32::YELLOW, value.0.to_string());
                        ui.label("/".to_string());
                        ui.colored_label(Color32::YELLOW, value.1.to_string());
                    });
                }

                let navigation = &submarine.navigation;
                add_info(ui, "Speed", navigation.speed);
                add_info(ui, "Acceleration", navigation.acceleration);
                add_info(ui, "Target", navigation.target);
                add_info(ui, "Position", navigation.position);

                ui.separator();

                let nav_control = compute_navigation(navigation);
                add_info(ui, "Target speed", nav_control.target_speed);
                add_info(ui, "Target acceleration", nav_control.target_acceleration);
                add_info(
                    ui,
                    "Target engine/pump speed",
                    nav_control.engine_and_pump_speed,
                );
            } else {
                ui.label("No submarine selected.");
            }

            if ui.button("Close").clicked() {
                *show_navigation_info = false;
            }
        });
    }

    if *show_update_settings {
        egui::Window::new("Update settings").show(ctx, |ui| {
            ui.checkbox(update_water, "Update water");
            ui.vertical(|ui| {
                ui.set_enabled(*update_water);
                ui.checkbox(enable_gravity, "Enable gravity");
                ui.checkbox(enable_inertia, "Enable inertia");
            });
            ui.checkbox(update_wires, "Update wires");
            ui.checkbox(update_sonar, "Update sonar");
            ui.checkbox(update_objects, "Update objects");
            ui.checkbox(update_position, "Update position");
            ui.checkbox(update_collision, "Update collision");

            if ui.button("Close").clicked() {
                *show_update_settings = false;
            }
        });
    }

    if *show_draw_settings {
        egui::Window::new("Draw settings").show(ctx, |ui| {
            ui.checkbox(draw_egui, "Draw egui widgets")
                .on_hover_text("Click the top-left gear button to re-enable the UI");
            ui.checkbox(draw_sea_dust, "Draw sea dust");
            ui.checkbox(draw_sea_caustics, "Draw sea caustics");
            ui.checkbox(draw_rocks, "Draw rocks");
            ui.checkbox(draw_background, "Draw background");
            ui.checkbox(draw_objects, "Draw objects");
            ui.checkbox(draw_walls, "Draw walls");
            ui.checkbox(draw_wires, "Draw wires");
            ui.checkbox(draw_water, "Draw water");
            ui.checkbox(draw_sonar, "Draw sonar");
            ui.checkbox(draw_engine_turbulence, "Draw engine turbulence");

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
            egui::ScrollArea::from_max_height(300.0).show(ui, |ui| {
                ui.label("This is a water simulation prototype used in the context of a game heavily inspired by Barotrauma.");
                ui.label("Zoom in on the sonar and click inside it to set a nagivation target; for now there's not much else to do.");
                ui.label("The code is here:");
                ui.hyperlink_to("https://github.com/andreivasiliu/cybersub", "https://github.com/andreivasiliu/cybersub");
                ui.label("If you like what you're seeing, I recommend checking out the game it's based on:");
                ui.hyperlink_to("https://github.com/Regalis11/Barotrauma", "https://github.com/Regalis11/Barotrauma");
                ui.label(
                    "It's called CyberSub because, for a few brief moments, I contemplated making the art be all cybespace-like to \
                    make up for my lack of artistic skill. I couldn't figure out why such realistic water would exist in cyberspace \
                    though, so I dropped the idea; but for now I don't have a better alternative so I'm sticking with it."
                );
                ui.label("Left-click to add water or interact with objects.");
                ui.label("On browsers, the right-click menu is disabled, in order to make scrolling easier. You can still shift-right-click.");
                ui.label(
                    "WASD, arrow keys, or hold right-click to move camera. Keypad +/- or scroll mouse-wheel to zoom. Minus doesn't \
                    work on browsers. No idea why. There is currently no way to move the camera with a touch-screen."
                );
                ui.label(
                    "Use the tool controls (Add Water, Add Walls, etc) at the bottom to switch what left-click paints. Holding \
                    shift or ctrl is a shortcut for switching."
                );
                ui.label(
                    "On some browsers, the sea dust shader is acting very strangely, with the dust specs looking much larger. \
                    No idea why."
                );
                ui.label("Firefox is having issues with rendering the whole thing; my phone and other browsers work fine though.");
                ui.label(
                    "If you're getting low FPS, disable the caustics shader, updating water and updating wires. I plan to revamp \
                    wires and optimize the other two, so it won't be a problem for long."
                );
            });

            if ui.button("Close").clicked() {
                *show_help = false;
            }
        });
    }
}
