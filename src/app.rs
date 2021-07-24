use crate::{draw::{Camera, draw_game, handle_keyboard_input, handle_pointer_input}, water::WaterGrid};

pub struct CyberSubApp {
    // Example stuff:
    label: String,
    grid: WaterGrid,
    show_total_water: bool,
    enable_gravity: bool,
    enable_inertia: bool,
    last_update: Option<f64>,
    show_ui: bool,
    show_help: bool,
    quit_game: bool,
    camera: Camera,
    error_message: Option<String>,
}

const WIDTH: usize = 300;
const HEIGHT: usize = 100;

impl Default for CyberSubApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello me!".to_owned(),
            grid: WaterGrid::new(WIDTH, HEIGHT),
            show_total_water: false,
            enable_gravity: true,
            enable_inertia: true,
            last_update: None,
            show_ui: true,
            show_help: false,
            quit_game: false,
            camera: Camera::default(),
            error_message: None,
        }
    }
}

impl CyberSubApp {
    pub fn update_game(&mut self, game_time: f64) {
        let Self {
            grid,
            enable_gravity,
            enable_inertia,
            last_update,
            ..
        } = self;

        if let Some(last_update) = last_update {
            let mut delta = (game_time - *last_update).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 30 updates per second, regardless of FPS
                delta -= 1.0 / 30.0;
                grid.update(*enable_gravity, *enable_inertia);
            }
        }
        *last_update = Some(game_time);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    pub fn draw_ui(&mut self, ctx: &egui::CtxRef) {
        let Self {
            label,
            grid,
            show_total_water,
            enable_gravity,
            enable_inertia,
            show_ui,
            show_help,
            quit_game,
            error_message,
            ..
        } = self;

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
                        egui::Hyperlink::new("https://github.com/not-fl3/macroquad/")
                            .text("macroquad"),
                    );
                    egui::warn_if_debug_build(ui);
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
        });

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
                ui.label("Left-click to add water, right-click to add walls, middle-click to remove walls.");
                ui.label("On browsers, right-click also opens the browser menu. I'm too lazy to fix that.");

                if ui.button("Close").clicked() {
                    *show_help = false;
                }
            });
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit_game
    }

    pub fn handle_pointer_input(&mut self) {
        handle_pointer_input(&mut self.grid, &mut self.camera);
    }

    pub fn handle_keyboard_input(&mut self) {
        handle_keyboard_input(&mut self.camera);
    }

    pub fn draw_game(&self) {
        draw_game(&self.grid, &self.camera);
    }
}

fn save(grid: &WaterGrid) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = std::fs::File::create("grid.bin")
            .map_err(|err| format!("Could not save: {}", err))?;
        let writer = std::io::BufWriter::new(file);

        bincode::serialize_into(writer, grid)
            .map_err(|err| format!("Could not serialize grid: {}", err))?;

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = grid;
        Err("Saving not yet possible on browsers".to_string())
    }
}

fn load() -> Result<WaterGrid, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let bytes = std::fs::File::open("grid.bin")
            .map_err(|err| format!("Could not load: {}", err))?;
        let reader = std::io::BufReader::new(bytes);

        let grid = bincode::deserialize_from(reader)
            .map_err(|err| format!("Could not deserialize: {}", err))?;

        Ok(grid)
    }

    #[cfg(target_arch = "wasm32")]
    {
        Err("Loading not yet possible on browsers".to_string())
    }
}