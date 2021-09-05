use egui::{
    plot::{Line, Plot, Value, Values},
    vec2, Align2, Button, Color32, Label, Slider, Ui,
};

use crate::{
    app::{GameSettings, NetworkSettings, PlacingObject, Tool},
    draw::DrawSettings,
    game_state::objects::{compute_navigation, OBJECT_TYPES},
    game_state::state::{GameState, UpdateSettings},
    game_state::update::Command,
    game_state::wires::WireColor,
    resources::MutableSubResources,
    saveload::{
        load_from_directory, load_template_from_data, save_to_directory, save_to_file_data,
    },
    Timings,
};

pub(crate) struct UiState {
    error_message: Option<String>,
    show_total_water: bool,
    show_bars: bool,
    show_main_settings: bool,
    show_toolbar: bool,
    show_help: bool,
    show_timings: bool,
    show_navigation_info: bool,
    show_draw_settings: bool,
    show_update_settings: bool,
    show_load_dialog: bool,
    show_save_dialog: bool,
    show_host_dialog: bool,
    show_join_dialog: bool,
    submarine_name: String,
    overwrite_save: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            error_message: None,
            show_total_water: false,
            show_bars: true,
            show_main_settings: true,
            show_toolbar: true,
            show_help: false,
            show_timings: false,
            show_navigation_info: false,
            show_draw_settings: false,
            show_update_settings: false,
            show_load_dialog: false,
            show_save_dialog: false,
            show_host_dialog: false,
            show_join_dialog: false,
            submarine_name: "NewSubmarine".to_string(),
            overwrite_save: false,
        }
    }
}

/// Called each time the UI needs repainting, which may be many times per second.
/// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
pub(crate) fn draw_ui(
    ctx: &egui::CtxRef,
    ui_state: &mut UiState,
    settings: &mut GameSettings,
    state: &GameState,
    mutable_sub_resources: &[MutableSubResources],
    timings: &Timings,
    commands: &mut Vec<Command>,
) {
    let UiState {
        error_message,
        show_total_water,
        show_bars,
        show_toolbar,
        show_main_settings,
        show_help,
        show_timings,
        show_navigation_info,
        show_draw_settings,
        show_update_settings,
        show_load_dialog,
        show_save_dialog,
        show_host_dialog,
        show_join_dialog,
        submarine_name,
        overwrite_save,
    } = ui_state;

    let GameSettings {
        draw_settings,
        network_settings,
        camera,
        current_submarine,
        current_tool,
        quit_game,
        placing_object,
        submarine_templates,
        ..
    } = settings;

    let GameState {
        submarines,
        update_settings,
        ..
    } = state;

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
        draw_shadows,
        debug_shadows,
    } = draw_settings;

    let mut new_update_settings = update_settings.clone();

    let UpdateSettings {
        update_water,
        enable_gravity,
        enable_inertia,
        update_wires,
        update_sonar,
        update_objects,
        update_position,
        update_collision,
    } = &mut new_update_settings;

    let NetworkSettings {
        server_tcp_address,
        server_ws_address,
        client_tcp_address,
        client_ws_address,
        start_server,
        server_started,
        connect_client,
        client_connected,
        network_status,
        network_error,
        download_progress,
    } = network_settings;

    if *show_bars {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Load submarine").clicked() {
                        *show_load_dialog = true;
                    }
                    if submarines.len() > *current_submarine {
                        ui.scope(|ui| {
                            ui.set_enabled(!cfg!(target_arch = "wasm32"));
                            if ui
                                .button("Save submarine")
                                .on_disabled_hover_text("Not available on browsers")
                                .clicked()
                            {
                                *show_save_dialog = true;
                            }
                        });

                        if ui.button("Clear water").clicked() {
                            commands.push(Command::ClearWater {
                                submarine_id: *current_submarine,
                            });
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
                    if ui.button("Show timings").clicked() {
                        *show_timings = !*show_timings;
                    }
                });
                egui::menu::menu(ui, "Objects", |ui| {
                    for (object_type_name, object_type) in OBJECT_TYPES {
                        if ui.button(object_type_name).clicked() {
                            *placing_object = Some(PlacingObject {
                                submarine: 0,
                                position: None,
                                object_type: object_type.clone(),
                            });
                        }
                    }
                });
                egui::menu::menu(ui, "Submarines", |ui| {
                    for (name, template) in submarine_templates.iter() {
                        if ui.button(name).clicked() {
                            commands.push(Command::CreateSubmarine {
                                submarine_template: Box::new(template.clone()),
                                rock_position: (100, 100),
                            });
                        }
                    }
                });
                egui::menu::menu(ui, "Network", |ui| {
                    ui.scope(|ui| {
                        ui.set_enabled(!cfg!(target_arch = "wasm32"));
                        let button = ui
                            .button("Host game")
                            .on_disabled_hover_text("Only available on the native client");
                        if button.clicked() {
                            *show_host_dialog = true;
                        }
                    });
                    if ui.button("Join game").clicked() {
                        *show_join_dialog = true;
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
                ui.label("FPS average:".to_string());
                ui.colored_label(Color32::GREEN, timings.fps_average.to_string());
                ui.label("x:".to_string());
                ui.colored_label(Color32::GREEN, camera.pointing_at.0.to_string());
                ui.label("y:".to_string());
                ui.colored_label(Color32::GREEN, camera.pointing_at.1.to_string());

                if let Some(submarine) = submarines.get(*current_submarine) {
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

    if *show_load_dialog {
        egui::Window::new("Load submarine")
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(submarine_name);
                });

                ui.horizontal(|ui| {
                    let load_button = Button::new("Load").enabled(!submarine_name.is_empty());

                    if ui.add(load_button).clicked() {
                        let mut load = || {
                            if cfg!(target_arch = "wasm32") {
                                Err("Not yet implemented on browsers".to_string())
                            } else {
                                let file_data = load_from_directory(submarine_name)?;
                                let template = load_template_from_data(file_data)?;
                                submarine_templates.push((submarine_name.to_owned(), template));
                                Ok(())
                            }
                        };

                        *error_message = if let Err(err) = load() {
                            Some(err)
                        } else {
                            Some(format!(
                                "Template '{}' added to Submarines menu.",
                                submarine_name
                            ))
                        };
                        *show_load_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        *show_load_dialog = false;
                    }
                });
            });
    }

    if *show_save_dialog {
        egui::Window::new("Save submarine")
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(submarine_name);
                });

                ui.checkbox(overwrite_save, "Overwrite existing files");

                ui.horizontal(|ui| {
                    let save_button = Button::new("Save").enabled(!submarine_name.is_empty());

                    if ui.add(save_button).clicked() {
                        let submarine = submarines.get(*current_submarine);
                        let resources = mutable_sub_resources.get(*current_submarine);

                        if let (Some(submarine), Some(resources)) = (submarine, resources) {
                            let save = || {
                                let file_data = save_to_file_data(submarine, resources)?;
                                save_to_directory(submarine_name, file_data, *overwrite_save)
                            };

                            if let Err(err) = save() {
                                *error_message = Some(err);
                            }
                        } else {
                            *error_message = Some("No submarine selected.".to_string());
                        }
                        *show_save_dialog = false;
                        *overwrite_save = false;
                    }
                    if ui.button("Cancel").clicked() {
                        *show_save_dialog = false;
                    }
                });
            });
    }

    if *show_host_dialog {
        egui::Window::new("Host game").show(ctx, |ui| {
            ui.scope(|ui| {
                ui.set_enabled(!*server_started && !*client_connected);

                ui.horizontal(|ui| {
                    ui.label("TCP address:");
                    ui.text_edit_singleline(server_tcp_address);
                });
                ui.horizontal(|ui| {
                    ui.label("WebSocket address:");
                    ui.text_edit_singleline(server_ws_address);
                });
                if ui.button("Start server").clicked() {
                    *start_server = true;
                }
            });

            ui.separator();

            ui.label(format!("Status: {}", network_status));
            if ui.button("Close").clicked() {
                *show_host_dialog = false;
            }
        });
    }

    if *show_join_dialog {
        egui::Window::new("Join game").show(ctx, |ui| {
            ui.scope(|ui| {
                ui.set_enabled(!*server_started && !*client_connected);

                ui.horizontal(|ui| {
                    ui.set_enabled(!cfg!(target_arch = "wasm32"));
                    ui.label("TCP address:");
                    ui.text_edit_singleline(client_tcp_address)
                        .on_disabled_hover_text("Not available on browser client");
                });
                ui.horizontal(|ui| {
                    ui.set_enabled(cfg!(target_arch = "wasm32"));
                    ui.label("WebSocket address:");
                    ui.text_edit_singleline(client_ws_address)
                        .on_disabled_hover_text("Only available on browser client");
                });
                ui.scope(|ui| {
                    let unavailable = if !cfg!(target_arch = "wasm32") {
                        "Only available on browser client"
                    } else if quad_url::path(false).starts_with("https://") {
                        "Cannot access ws:// when the page is loaded from an https:// URL \
                        (such as from Github Pages), and wss:// is not yet supported by \
                        the server. For now, load the page from an http:// location instead."
                    } else {
                        "Already connected"
                    };
                    ui.set_enabled(unavailable.is_empty());

                    if ui
                        .button("Connect to server")
                        .on_disabled_hover_text(unavailable)
                        .clicked()
                    {
                        *connect_client = true;
                    }
                })
            });

            ui.separator();

            ui.label(format!("Status: {}", network_status));
            if let Some(error) = network_error {
                ui.horizontal(|ui| {
                    ui.label("Error:");
                    ui.colored_label(Color32::RED, error.as_str());
                });
            }
            if ui.button("Close").clicked() {
                *show_join_dialog = false;
            }

            if let Some(progress) = download_progress {
                ui.separator();
                let progress = *progress as f32 / 100.0;
                ui.add(egui::ProgressBar::new(progress).animate(true));
            }
        });
    }

    if *show_main_settings {
        egui::Window::new("Settings").show(ctx, |ui| {
            ui.collapsing("Show windows", |ui| {
                ui.checkbox(show_toolbar, "Show toolbar");
                ui.checkbox(show_main_settings, "Show main settings");
                ui.checkbox(show_navigation_info, "Show navigation info");
                ui.checkbox(show_draw_settings, "Show draw settings");
                ui.checkbox(show_update_settings, "Show update settings");
                ui.checkbox(show_timings, "Show timings");
            });
            ui.collapsing("Performance settings", |ui| {
                ui.checkbox(draw_sea_caustics, "Draw caustics");
                ui.checkbox(draw_water, "Draw water");
                ui.checkbox(update_water, "Update water")
                    .on_hover_text("Warning: this will lock the submarine's vertical acceleration");
                ui.checkbox(draw_egui, "Draw UI")
                    .on_hover_text("Click the top-left gear button to re-enable the UI");
            });
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
                if placing_object.is_some() {
                    ui.label("Left-click to place object. Right-click to cancel. Hold shift to place more objects.");
                    if ui.button("Cancel").clicked() {
                        *placing_object = None;
                    }
                } else if let Tool::Interact = current_tool {
                    ui.radio_value(current_tool, Tool::Interact, "Interact");
                    ui.radio_value(current_tool, Tool::EditWater { add: true }, "Edit Water");
                    ui.radio_value(current_tool, Tool::EditWalls { add: true }, "Edit Walls");
                    ui.radio_value(current_tool, Tool::EditWires { color: WireColor::Brown }, "Edit Wires");
                } else if let Tool::EditWater { add } = current_tool {
                    ui.label("Edit water:");
                    ui.radio_value(add, true, "Add");
                    ui.radio_value(add, false, "Remove");
                    if ui.button("Cancel").clicked() {
                        *current_tool = Tool::Interact
                    }
                } else if let Tool::EditWalls { add } = current_tool {
                    ui.label("Edit walls:");
                    ui.radio_value(add, true, "Add");
                    ui.radio_value(add, false, "Remove");
                    if ui.button("Cancel").clicked() {
                        *current_tool = Tool::Interact
                    }
                } else if let Tool::EditWires { color } = current_tool {
                    ui.label("Edit wires:");
                    ui.radio_value(color, WireColor::Purple, "Purple");
                    ui.radio_value(color, WireColor::Brown, "Brown");
                    ui.radio_value(color, WireColor::Blue, "Blue");
                    ui.radio_value(color, WireColor::Green, "Green");
                    if ui.button("Cancel").clicked() {
                        *current_tool = Tool::Interact
                    }
                }
            });
        });
    }

    if *show_navigation_info {
        egui::Window::new("Navigation info").show(ctx, |ui| {
            if let Some(submarine) = submarines.get(*current_submarine) {
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
            ui.checkbox(draw_shadows, "Draw shadows");

            ui.checkbox(debug_shadows, "Debug shadows");

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

            if !cfg!(target_arch = "wasm32") {
                show_timer("egui_layout", timings.egui_layout);
                show_timer("egui_drawing", timings.egui_drawing);
                show_timer("input_handling", timings.input_handling);
                show_timer("game_update", timings.game_update);
                show_timer("game_layout", timings.game_layout);
                show_timer("frame_update", timings.frame_update);
            }
            show_timer("FPS", timings.fps);
            show_timer("FPS average", timings.fps_average);

            ui.collapsing("Graphs", |ui| {
                let first_timing = timings.fps_history.front().map(|(x, _y)| *x).unwrap_or(0.0);

                ui.label("FPS:");
                let fps_line = Line::new(Values::from_values_iter(
                    timings
                        .fps_history
                        .iter()
                        .map(|(x, y)| Value::new(*x - first_timing, *y)),
                ));
                let plot = Plot::new("FPS")
                    .line(fps_line)
                    .width(200.0)
                    .height(100.0)
                    .show_x(false)
                    .include_x(0.0)
                    .include_x(1.0)
                    .include_y(0.0)
                    .include_y(144.0);
                ui.add(plot);

                ui.label("FPS average:");
                let fps_average_line = Line::new(Values::from_values_iter(
                    timings
                        .fps_average_history
                        .iter()
                        .map(|(x, y)| Value::new(*x - first_timing, *y)),
                ));
                let plot = Plot::new("Average FPS")
                    .line(fps_average_line)
                    .width(200.0)
                    .height(100.0)
                    .show_x(false)
                    .include_x(0.0)
                    .include_x(1.0)
                    .include_y(0.0)
                    .include_y(144.0);
                ui.add(plot);
            });

            if ui.button("Close").clicked() {
                *show_timings = false;
            }
        });
    }

    if error_message.is_some() {
        egui::Window::new("Error")
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ctx, |ui| {
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
                ui.label("Left-click to interact with objects, hold LMB to drag camera. RMB can also drag camera regardless of the current tool.");
                ui.label("On browsers, the right-click menu is disabled, in order to make scrolling easier. You can still shift-right-click.");
                ui.label(
                    "Regardless of the selected tool, you can use WASD, arrow keys, or hold the right mouse button to move camera."
                );
                ui.label(
                    "Use the tool controls (Add Water, Add Walls, etc) at the bottom to switch what left-click does."
                );
                ui.label(
                    "On some devices, the sea dust shader is acting very strangely, with the dust specs looking much larger. \
                    This is related to the float precision (highp vs lowp). It can be fixed, just need to figure out how."
                );
                ui.label(
                    "If you're getting low FPS, disable the caustics shader and/or updating water. I plan to revamp \
                    or optimize them, so it won't be a problem for long."
                );
            });

            if ui.button("Close").clicked() {
                *show_help = false;
            }
        });
    }

    if new_update_settings != *update_settings {
        commands.push(Command::ChangeUpdateSettings {
            update_settings: new_update_settings,
        });
    }
}
