use crate::{
    draw::{draw_game, Camera},
    input::{handle_keyboard_input, handle_pointer_input},
    objects::{update_objects, Object},
    saveload::{load_objects, load_png_from_bytes, load_wires},
    ui::{draw_ui, UiState},
    water::WaterGrid,
    wires::WireGrid,
    Resources,
};

pub struct CyberSubApp {
    ui_state: UiState,
    game_state: GameState,
    game_settings: GameSettings,
    pub timings: Timings,
}

pub(crate) struct GameSettings {
    pub enable_gravity: bool,
    pub enable_inertia: bool,
    pub camera: Camera,
    pub current_tool: Tool,
    pub draw_sea_water: bool,
    pub draw_objects: bool,
    pub draw_water_grid: bool,
    pub quit_game: bool,
    pub dragging_object: bool,
    pub highlighting_object: Option<(usize, bool)>,
}

pub(crate) struct GameState {
    pub last_update: Option<f64>,
    pub water_grid: WaterGrid,
    pub wire_grid: WireGrid,
    pub objects: Vec<Object>,
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
                current_tool: Tool::AddWater,
                quit_game: false,
                draw_sea_water: true,
                draw_objects: true,
                draw_water_grid: true,
                dragging_object: false,
                highlighting_object: None,
            },
            game_state: GameState {
                water_grid: WaterGrid::new(WIDTH, HEIGHT),
                wire_grid: WireGrid::new(WIDTH, HEIGHT),
                objects: Vec::new(),
                last_update: None,
            },
            ui_state: UiState::default(),
            timings: Timings::default(),
        }
    }
}

impl CyberSubApp {
    pub fn load_grid(&mut self, grid_bytes: Vec<u8>) {
        self.game_state.water_grid = load_png_from_bytes(&grid_bytes).expect("Could not load grid");
        let (width, height) = self.game_state.water_grid.size();
        self.game_state.wire_grid = WireGrid::new(width, height);
    }

    pub fn load_objects(&mut self) {
        self.game_state.objects = load_objects();
        load_wires(&mut self.game_state.wire_grid);
    }

    pub fn update_game(&mut self, game_time: f64) {
        let last_update = &mut self.game_state.last_update;

        if let Some(last_update) = last_update {
            let mut delta = (game_time - *last_update).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 30 updates per second, regardless of FPS
                delta -= 1.0 / 30.0;
                self.game_state.water_grid.update(
                    self.game_settings.enable_gravity,
                    self.game_settings.enable_inertia,
                );
                for _ in 0..3 {
                    self.game_state.wire_grid.update();
                }
                update_objects(
                    &mut self.game_state.objects,
                    &mut self.game_state.water_grid,
                    &mut self.game_state.wire_grid,
                );
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
            &self.timings,
        );
    }

    pub fn should_quit(&self) -> bool {
        self.game_settings.quit_game
    }

    pub fn handle_pointer_input(&mut self) {
        handle_pointer_input(
            &mut self.game_state.water_grid,
            &mut self.game_state.wire_grid,
            &mut self.game_state.objects,
            &mut self.game_settings.camera,
            &self.game_settings.current_tool,
            &mut self.game_settings.dragging_object,
            &mut self.game_settings.highlighting_object,
        );
    }

    pub fn handle_keyboard_input(&mut self) {
        handle_keyboard_input(
            &mut self.game_settings.camera,
            &mut self.game_settings.current_tool,
        );
    }

    pub fn draw_game(&self, resources: &Resources) {
        draw_game(
            &self.game_state.water_grid,
            &self.game_state.wire_grid,
            &self.game_settings.camera,
            self.game_settings.draw_sea_water,
            self.game_settings.draw_objects,
            self.game_settings.draw_water_grid,
            &self.game_state.objects,
            resources,
            &self.game_settings.highlighting_object,
        );
    }
}
