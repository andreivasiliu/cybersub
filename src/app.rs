use crate::{
    draw::{draw_game, Camera, DrawSettings},
    game_state::objects::ObjectType,
    game_state::sonar::Sonar,
    game_state::state::{GameState, Navigation, SubmarineState},
    game_state::update::{update_game, Command, SubmarineUpdatedEvent, UpdateEvent},
    game_state::wires::WireColor,
    input::{handle_keyboard_input, handle_pointer_input},
    resources::{MutableResources, MutableSubResources, Resources},
    saveload::{load_from_file_data, load_rocks_from_png, save_to_file_data, SubmarineData},
    ui::{draw_ui, UiState},
    SubmarineFileData,
};

pub struct CyberSubApp {
    pub timings: Timings,
    ui_state: UiState,
    game_state: GameState,
    game_settings: GameSettings,
    commands: Vec<Command>,
    update_events: Vec<UpdateEvent>,
    resources: Resources,
    mutable_resources: MutableResources,
    mutable_sub_resources: Vec<MutableSubResources>,
}

pub(crate) struct GameSettings {
    pub draw_settings: DrawSettings,
    pub camera: Camera,
    pub current_submarine: usize,
    pub current_tool: Tool,
    pub quit_game: bool,
    pub dragging_object: bool,
    pub highlighting_settings: bool,
    pub last_draw: Option<f64>,
    pub animation_ticks: u32,
    pub add_submarine: Option<usize>,
    pub placing_object: Option<PlacingObject>,
    pub submarine_templates: Vec<(String, SubmarineData)>,
}

#[derive(PartialEq, Eq)]
pub(crate) enum Tool {
    Interact,
    EditWater { add: bool },
    EditWalls { add: bool },
    EditWires { color: WireColor },
}

#[derive(Default)]
pub struct Timings {
    pub egui_layout: u32,
    pub egui_drawing: u32,
    pub input_handling: u32,
    pub game_update: u32,
    pub game_layout: u32,
    pub frame_update: u32,
    pub fps: u32,
    pub frame_time: u32,
}

pub(crate) struct PlacingObject {
    pub submarine: usize,
    pub position: Option<(usize, usize)>,
    pub object_type: ObjectType,
}

impl Default for CyberSubApp {
    fn default() -> Self {
        let draw_settings = DrawSettings {
            draw_egui: true,
            draw_sea_dust: true,
            draw_sea_caustics: true,
            draw_rocks: true,
            draw_background: true,
            draw_objects: true,
            draw_walls: true,
            draw_wires: true,
            draw_water: true,
            draw_sonar: true,
            draw_engine_turbulence: true,
        };

        Self {
            timings: Timings::default(),
            game_settings: GameSettings {
                draw_settings,
                camera: Camera {
                    zoom: -200,
                    ..Default::default()
                },
                current_submarine: 0,
                current_tool: Tool::Interact,
                quit_game: false,
                dragging_object: false,
                highlighting_settings: false,
                last_draw: None,
                animation_ticks: 0,
                add_submarine: None,
                placing_object: None,
                submarine_templates: Vec::new(),
            },
            commands: Vec::new(),
            update_events: Vec::new(),
            game_state: GameState::default(),
            ui_state: UiState::default(),
            resources: Resources::new(),
            mutable_resources: MutableResources::new(),
            mutable_sub_resources: Vec::new(),
        }
    }
}

impl CyberSubApp {
    fn load_submarine(&mut self, template: SubmarineData) -> Result<(), String> {
        let SubmarineData {
            water_grid,
            background,
            objects,
            wire_grid,
        } = template;
        let (width, height) = water_grid.size();

        // Middle of the world
        let (rock_width, rock_height) = self.game_state.rock_grid.size();
        let (middle_x, middle_y) = (
            (rock_width as i32 / 2) * 16 * 16,
            (rock_height as i32 / 2) * 16 * 16,
        );

        // Put the middle of the sub at the middle of the world
        let (pos_x, pos_y) = (
            middle_x - width as i32 * 16 / 2,
            middle_y - height as i32 * 16 / 2,
        );

        // Change camera to its middle and set it as current
        self.game_settings.current_submarine = self.game_state.submarines.len();
        self.game_settings.camera.offset_x = -(width as f32) / 2.0;
        self.game_settings.camera.offset_y = -(height as f32) / 2.0;

        self.game_state.submarines.push(SubmarineState {
            water_grid,
            wire_grid,
            objects,
            navigation: Navigation {
                position: (pos_x, pos_y),
                target: (pos_x, pos_y),
                ..Default::default()
            },
            sonar: Sonar::default(),
            collisions: Vec::new(),
        });

        self.mutable_sub_resources
            .push(MutableSubResources::new(background));

        Ok(())
    }

    pub fn load_submarine_template(
        &mut self,
        name: impl Into<String>,
        file_data: SubmarineFileData,
    ) -> Result<usize, String> {
        let template = load_from_file_data(file_data)?;
        self.game_settings
            .submarine_templates
            .push((name.into(), template));
        Ok(self.game_settings.submarine_templates.len() - 1)
    }

    pub fn add_submarine(&mut self, template_index: usize) {
        self.game_settings.add_submarine = Some(template_index);
    }

    pub fn save_submarines(&mut self) -> Result<SubmarineFileData, String> {
        let current_submarine = self.game_settings.current_submarine;
        let submarine = self.game_state.submarines.get(current_submarine);
        let resources = self.mutable_sub_resources.get(current_submarine);

        if let (Some(submarine), Some(resources)) = (submarine, resources) {
            return save_to_file_data(submarine, resources);
        }

        Err("No submarine selected".to_string())
    }

    pub fn load_rocks(&mut self, world_bytes: &[u8]) {
        self.game_state.rock_grid = load_rocks_from_png(world_bytes);
    }

    pub fn update_game(&mut self, game_time: f64) {
        if let Some(template_id) = self.game_settings.add_submarine {
            self.game_settings.add_submarine = None;
            let submarine = self
                .game_settings
                .submarine_templates
                .get(template_id)
                .expect("Template was requested this frame")
                .clone();

            // FIXME: This should be done through a command somehow
            self.load_submarine(submarine.1).ok();
        }

        self.game_settings.animation_ticks = 0;

        if let Some(last_draw) = self.game_settings.last_draw {
            let mut delta = (game_time - last_draw).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 60 animation updates per second, regardless of FPS

                delta -= 1.0 / 60.0;
                self.game_settings.animation_ticks += 1;
            }
        }

        if let Some(last_update) = &mut self.game_state.last_update {
            let mut delta = (game_time - *last_update).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 30 updates per second, regardless of FPS
                delta -= 1.0 / 30.0;

                self.update_events.clear();

                update_game(
                    &self.commands,
                    &mut self.game_state,
                    &mut self.update_events,
                );

                self.commands.clear();

                for event in &self.update_events {
                    match event {
                        UpdateEvent::Submarine {
                            submarine_id,
                            submarine_event,
                        } => {
                            let mutable_sub_resources = self.mutable_sub_resources.get_mut(*submarine_id).expect("All submarines should have their own MutableSubResources instance");

                            match submarine_event {
                                SubmarineUpdatedEvent::Sonar => {
                                    mutable_sub_resources.sonar_updated = true;
                                }
                                SubmarineUpdatedEvent::Walls => {
                                    mutable_sub_resources.walls_updated = true;
                                }
                                SubmarineUpdatedEvent::Wires => {
                                    mutable_sub_resources.wires_updated = true;
                                }
                                SubmarineUpdatedEvent::Signals => {
                                    mutable_sub_resources.signals_updated = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.game_settings.last_draw = Some(game_time);
        self.game_state.last_update = Some(game_time);

        // Follow submarine with camera
        let submarine_camera = self
            .game_state
            .submarines
            .get(self.game_settings.current_submarine)
            .map(|submarine| {
                (
                    submarine.navigation.position.0,
                    submarine.navigation.position.1,
                )
            });

        self.game_settings.camera.current_submarine = submarine_camera;
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    pub fn draw_ui(&mut self, ctx: &egui::CtxRef) {
        if self.game_settings.draw_settings.draw_egui {
            draw_ui(
                ctx,
                &mut self.ui_state,
                &mut self.game_settings,
                &self.game_state,
                &self.mutable_sub_resources,
                &self.timings,
                &mut self.commands,
            );
        }
    }

    pub fn should_quit(&self) -> bool {
        self.game_settings.quit_game
    }

    pub fn handle_pointer_input(&mut self) {
        for (sub_index, submarine) in &mut self.game_state.submarines.iter().enumerate() {
            let mutable_resources = self
                .mutable_sub_resources
                .get_mut(sub_index)
                .expect("All submarines should have a MutableSubResources instance");
            handle_pointer_input(
                &mut self.commands,
                submarine,
                sub_index,
                mutable_resources,
                &mut self.game_settings,
            );
        }
    }

    pub fn handle_keyboard_input(&mut self) {
        handle_keyboard_input(&mut self.game_settings.camera);
    }

    pub fn draw_game(&mut self) {
        draw_game(
            &self.game_state,
            &self.game_settings,
            &self.timings,
            &self.resources,
            &mut self.mutable_resources,
            &mut self.mutable_sub_resources,
        );
    }
}
