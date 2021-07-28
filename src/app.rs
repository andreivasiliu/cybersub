use crate::{
    draw::{draw_game, Camera},
    input::{handle_keyboard_input, handle_pointer_input},
    objects::{update_objects, Object},
    saveload::{load_objects, load_png_from_bytes},
    ui::{draw_ui, UiState},
    water::WaterGrid,
    Resources,
};

pub struct CyberSubApp {
    ui_state: UiState,
    game_state: GameState,
    game_settings: GameSettings,
}

pub(crate) struct GameSettings {
    pub enable_gravity: bool,
    pub enable_inertia: bool,
    pub camera: Camera,
    pub current_tool: Tool,
    pub draw_sea_water: bool,
    pub quit_game: bool,
    pub dragging_object: bool,
}

pub(crate) struct GameState {
    pub last_update: Option<f64>,
    pub grid: WaterGrid,
    pub objects: Vec<Object>,
}

#[derive(PartialEq, Eq)]
pub(crate) enum Tool {
    AddWater,
    AddWalls,
    RemoveWalls,
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
                dragging_object: false,
            },
            game_state: GameState {
                grid: WaterGrid::new(WIDTH, HEIGHT),
                objects: Vec::new(),
                last_update: None,
            },
            ui_state: UiState::default(),
        }
    }
}

impl CyberSubApp {
    pub fn load_grid(&mut self, grid_bytes: Vec<u8>) {
        self.game_state.grid = load_png_from_bytes(&grid_bytes).expect("Could not load grid");
    }

    pub fn load_objects(&mut self) {
        self.game_state.objects = load_objects();
    }

    pub fn update_game(&mut self, game_time: f64) {
        let last_update = &mut self.game_state.last_update;

        if let Some(last_update) = last_update {
            let mut delta = (game_time - *last_update).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 30 updates per second, regardless of FPS
                delta -= 1.0 / 30.0;
                self.game_state.grid.update(
                    self.game_settings.enable_gravity,
                    self.game_settings.enable_inertia,
                );
                update_objects(&mut self.game_state.objects, &mut self.game_state.grid);
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
        );
    }

    pub fn should_quit(&self) -> bool {
        self.game_settings.quit_game
    }

    pub fn handle_pointer_input(&mut self) {
        handle_pointer_input(
            &mut self.game_state.grid,
            &mut self.game_state.objects,
            &mut self.game_settings.camera,
            &self.game_settings.current_tool,
            &mut self.game_settings.dragging_object,
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
            &self.game_state.grid,
            &self.game_settings.camera,
            self.game_settings.draw_sea_water,
            &self.game_state.objects,
            resources,
        );
    }
}
