use crate::{
    draw::{draw_game, Camera, DrawSettings},
    input::{handle_keyboard_input, handle_pointer_input},
    objects::{update_objects, Object},
    resources::MutableResources,
    rocks::RockGrid,
    saveload::{load_objects, load_png_from_bytes, load_rocks_from_png, load_wires},
    ui::{draw_ui, UiState},
    water::WaterGrid,
    wires::WireGrid,
    Resources,
};

pub struct CyberSubApp {
    ui_state: UiState,
    game_state: GameState,
    game_settings: GameSettings,
    draw_settings: DrawSettings,
    pub timings: Timings,
}

pub(crate) struct GameSettings {
    pub enable_gravity: bool,
    pub enable_inertia: bool,
    pub camera: Camera,
    pub current_submarine: usize,
    pub current_tool: Tool,
    pub quit_game: bool,
    pub dragging_object: bool,
    pub highlighting_object: Option<(usize, bool)>,
}

pub(crate) struct GameState {
    pub last_update: Option<f64>,
    pub rock_grid: RockGrid,
    pub submarines: Vec<SubmarineState>,
}

pub(crate) struct SubmarineState {
    pub water_grid: WaterGrid,
    pub wire_grid: WireGrid,
    pub objects: Vec<Object>,
    pub position: (i32, i32),
    pub speed: (i32, i32),
    pub acceleration: (i32, i32),
}

#[derive(PartialEq, Eq)]
pub(crate) enum Tool {
    AddWater,
    AddWall,
    AddOrangeWire,
    AddBrownWire,
    AddBlueWire,
    AddGreenWire,
    RemoveWall,
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

const WIDTH: usize = 300;
const HEIGHT: usize = 100;

impl Default for CyberSubApp {
    fn default() -> Self {
        Self {
            game_settings: GameSettings {
                enable_gravity: true,
                enable_inertia: true,
                camera: Camera {
                    zoom: -200,
                    ..Default::default()
                },
                current_submarine: 0,
                current_tool: Tool::AddWater,
                quit_game: false,
                dragging_object: false,
                highlighting_object: None,
            },
            game_state: GameState {
                last_update: None,
                rock_grid: RockGrid::new(WIDTH, HEIGHT),
                submarines: Vec::new(),
            },
            draw_settings: DrawSettings {
                draw_sea: true,
                draw_rocks: true,
                draw_objects: true,
                draw_walls: true,
                draw_wires: true,
                draw_water: true,
            },
            ui_state: UiState::default(),
            timings: Timings::default(),
        }
    }
}

impl CyberSubApp {
    pub fn load_submarine(&mut self, grid_bytes: &[u8]) {
        let water_grid = load_png_from_bytes(&grid_bytes).expect("Could not load grid");
        let (width, height) = water_grid.size();
        let wire_grid = load_wires(width, height);
        let objects = load_objects();

        self.game_state.submarines.push(SubmarineState {
            water_grid,
            wire_grid,
            objects,
            position: (0, 0),
            speed: (0, 0),
            acceleration: (0, 0),
        });
    }

    pub fn load_rocks(&mut self, world_bytes: &[u8]) {
        self.game_state.rock_grid = load_rocks_from_png(world_bytes);
    }

    pub fn update_game(&mut self, game_time: f64) {
        let last_update = &mut self.game_state.last_update;

        if let Some(last_update) = last_update {
            let mut delta = (game_time - *last_update).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 30 updates per second, regardless of FPS
                delta -= 1.0 / 30.0;

                for submarine in &mut self.game_state.submarines {
                    submarine.acceleration.1 = ((submarine.water_grid.total_water() as i32 - 1_500_000) as f32 / 3_000_00.0 / 1.0) as i32;

                    submarine.speed.0 = (submarine.speed.0 + submarine.acceleration.0).clamp(-1024, 1024);
                    submarine.speed.1 = (submarine.speed.1 + submarine.acceleration.1).clamp(-1024, 1024);

                    submarine.position.0 += submarine.speed.0 / 256;
                    submarine.position.1 += submarine.speed.1 / 256;

                    submarine.water_grid.update(
                        self.game_settings.enable_gravity,
                        self.game_settings.enable_inertia,
                    );
                    for _ in 0..2 {
                        submarine.wire_grid.update();
                    }
                    update_objects(
                        &mut submarine.objects,
                        &mut submarine.water_grid,
                        &mut submarine.wire_grid,
                    );
                }

                let submarine_camera = self.game_state
                    .submarines
                    .get(self.game_settings.current_submarine)
                    .map(|submarine| submarine.position);

                self.game_settings.camera.current_submarine = submarine_camera;
            }
        }
        *last_update = Some(game_time);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    pub fn draw_ui(&mut self, ctx: &egui::CtxRef) {
        draw_ui(
            ctx,
            &mut self.ui_state,
            &mut self.game_settings,
            &mut self.game_state,
            &mut self.draw_settings,
            &self.timings,
        );
    }

    pub fn should_quit(&self) -> bool {
        self.game_settings.quit_game
    }

    pub fn handle_pointer_input(&mut self) {
        for submarine in &mut self.game_state.submarines {
            handle_pointer_input(
                submarine,
                &mut self.game_settings.camera,
                &self.game_settings.current_tool,
                &mut self.game_settings.dragging_object,
                &mut self.game_settings.highlighting_object,
            );
        }
    }

    pub fn handle_keyboard_input(&mut self) {
        handle_keyboard_input(
            &mut self.game_settings.camera,
            &mut self.game_settings.current_tool,
        );
    }

    pub fn draw_game(&self, resources: &Resources, mutable_resources: &mut MutableResources) {
        draw_game(
            &self.game_state.submarines,
            &self.game_state.rock_grid,
            &self.game_settings.camera,
            &self.draw_settings,
            resources,
            mutable_resources,
            &self.game_settings.highlighting_object,
        );
    }
}
