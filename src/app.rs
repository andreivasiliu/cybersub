use crate::{
    collisions::{update_rock_collisions, update_submarine_collisions},
    draw::{draw_game, Camera, DrawSettings},
    input::{handle_keyboard_input, handle_pointer_input},
    objects::{update_objects, Object},
    resources::{MutableResources, MutableSubResources},
    rocks::RockGrid,
    saveload::{load_objects, load_png_from_bytes, load_rocks_from_png, load_wires},
    sonar::{find_visible_edge_cells, Sonar},
    ui::{draw_ui, UiState},
    water::WaterGrid,
    wires::WireGrid,
    Resources,
};

pub struct CyberSubApp {
    pub timings: Timings,
    ui_state: UiState,
    game_state: GameState,
    game_settings: GameSettings,
    draw_settings: DrawSettings,
    update_settings: UpdateSettings,
    mutable_resources: MutableResources,
    mutable_sub_resources: Vec<MutableSubResources>,
}

pub(crate) struct GameSettings {
    pub enable_gravity: bool,
    pub enable_inertia: bool,
    pub camera: Camera,
    pub current_submarine: usize,
    pub current_tool: Tool,
    pub quit_game: bool,
    pub dragging_object: bool,
    pub highlighting_settings: bool,
    pub last_draw: Option<f64>,
    pub animation_ticks: u32,
    pub add_submarine: bool,
}

pub(crate) struct UpdateSettings {
    pub update_water: bool,
    pub update_wires: bool,
    pub update_sonar: bool,
    pub update_objects: bool,
    pub update_position: bool,
    pub update_collision: bool,
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
    pub sonar: Sonar,
    pub navigation: Navigation,
}

#[derive(Default)]
pub(crate) struct Navigation {
    pub target: (i32, i32),
    pub position: (i32, i32),
    pub speed: (i32, i32),
    pub acceleration: (i32, i32),
}

#[derive(PartialEq, Eq)]
pub(crate) enum Tool {
    AddWater,
    AddWall,
    AddPurpleWire,
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
            timings: Timings::default(),
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
                highlighting_settings: false,
                last_draw: None,
                animation_ticks: 0,
                add_submarine: false,
            },
            game_state: GameState {
                last_update: None,
                rock_grid: RockGrid::new(WIDTH, HEIGHT),
                submarines: Vec::new(),
            },
            draw_settings: DrawSettings {
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
            },
            update_settings: UpdateSettings {
                update_water: true,
                update_wires: true,
                update_sonar: true,
                update_objects: true,
                update_position: true,
                update_collision: true,
            },
            ui_state: UiState::default(),
            mutable_resources: MutableResources::new(),
            mutable_sub_resources: Vec::new(),
        }
    }
}

impl CyberSubApp {
    pub fn load_submarine(&mut self, grid_bytes: &[u8]) {
        let water_grid = load_png_from_bytes(grid_bytes).expect("Could not load grid");
        let (width, height) = water_grid.size();
        let wire_grid = load_wires(width, height);
        let objects = load_objects();

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

        // If adding the main submarine, also change camera to its middle
        if self.game_state.submarines.len() == self.game_settings.current_submarine {
            self.game_settings.camera.offset_x = -(width as f32) / 2.0;
            self.game_settings.camera.offset_y = -(height as f32) / 2.0;
        }

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
        });

        self.mutable_sub_resources.push(MutableSubResources::new());
    }

    pub fn load_rocks(&mut self, world_bytes: &[u8]) {
        self.game_state.rock_grid = load_rocks_from_png(world_bytes);
    }

    pub fn update_game(&mut self, game_time: f64) {
        if self.game_settings.add_submarine {
            self.game_settings.add_submarine = false;
            self.load_submarine(include_bytes!("../grid.png"));
        }

        let last_update = &mut self.game_state.last_update;
        let last_draw = &mut self.game_settings.last_draw;
        self.game_settings.animation_ticks = 0;

        if let Some(last_draw) = last_draw {
            let mut delta = (game_time - *last_draw).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 60 animation updates per second, regardless of FPS

                delta -= 1.0 / 60.0;
                self.game_settings.animation_ticks += 1;
            }
        }

        if let Some(last_update) = last_update {
            let mut delta = (game_time - *last_update).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 30 updates per second, regardless of FPS
                delta -= 1.0 / 30.0;
                self.game_settings.animation_ticks += 1;

                self.mutable_resources.collisions.clear();

                for (sub_index, submarine) in &mut self.game_state.submarines.iter_mut().enumerate()
                {
                    let mutable_resources = self
                        .mutable_sub_resources
                        .get_mut(sub_index)
                        .expect("All submarines should have a MutableSubResources instance");

                    if self.update_settings.update_position {
                        let navigation = &mut submarine.navigation;
                        navigation.acceleration.1 =
                            ((submarine.water_grid.total_water() as i32 - 1_500_000) as f32
                                / (3_000_000.0 / 10.0)) as i32;

                        navigation.speed.0 =
                            (navigation.speed.0 + navigation.acceleration.0).clamp(-2048, 2048);
                        navigation.speed.1 =
                            (navigation.speed.1 + navigation.acceleration.1).clamp(-2048, 2048);

                        navigation.position.0 += navigation.speed.0 / 256;
                        navigation.position.1 += navigation.speed.1 / 256;
                    }

                    if self.update_settings.update_water {
                        submarine.water_grid.update(
                            self.game_settings.enable_gravity,
                            self.game_settings.enable_inertia,
                        );
                    }
                    if self.update_settings.update_wires {
                        for _ in 0..3 {
                            submarine
                                .wire_grid
                                .update(&mut mutable_resources.signals_updated);
                        }
                    }
                    if self.update_settings.update_objects {
                        update_objects(submarine);
                    }
                    if self.update_settings.update_sonar {
                        update_sonar(
                            &mut submarine.sonar,
                            &submarine.navigation,
                            submarine.water_grid.size(),
                            &self.game_state.rock_grid,
                            mutable_resources,
                        );
                    }

                    if self.update_settings.update_collision {
                        mutable_resources.collisions.clear();
                        update_rock_collisions(
                            &submarine.water_grid,
                            &self.game_state.rock_grid,
                            &submarine.navigation,
                            &mut self.mutable_resources,
                            mutable_resources,
                        );
                    }
                }

                for (sub1_index, submarine1) in self.game_state.submarines.iter().enumerate() {
                    for (sub2_index, submarine2) in self.game_state.submarines.iter().enumerate() {
                        if sub1_index == sub2_index {
                            continue;
                        }

                        let mutable_resources = self
                            .mutable_sub_resources
                            .get_mut(sub1_index)
                            .expect("All submarines should have a MutableSubResources instance");

                        update_submarine_collisions(
                            &submarine1.water_grid,
                            &submarine2.water_grid,
                            &submarine1.navigation,
                            &submarine2.navigation,
                            mutable_resources,
                        );
                    }
                }

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
        }
        *last_draw = Some(game_time);
        *last_update = Some(game_time);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    pub fn draw_ui(&mut self, ctx: &egui::CtxRef) {
        if self.draw_settings.draw_egui {
            draw_ui(
                ctx,
                &mut self.ui_state,
                &mut self.game_settings,
                &mut self.game_state,
                &mut self.draw_settings,
                &mut self.update_settings,
                &self.timings,
            );
        }
    }

    pub fn should_quit(&self) -> bool {
        self.game_settings.quit_game
    }

    pub fn handle_pointer_input(&mut self) {
        for (sub_index, submarine) in &mut self.game_state.submarines.iter_mut().enumerate() {
            let mutable_resources = self
                .mutable_sub_resources
                .get_mut(sub_index)
                .expect("All submarines should have a MutableSubResources instance");
            handle_pointer_input(
                submarine,
                mutable_resources,
                &mut self.game_settings,
                &mut self.draw_settings.draw_egui,
            );
        }
    }

    pub fn handle_keyboard_input(&mut self) {
        handle_keyboard_input(
            &mut self.game_settings.camera,
            &mut self.game_settings.current_tool,
        );
    }

    pub fn draw_game(&mut self, resources: &Resources) {
        draw_game(
            &self.game_state,
            &self.game_settings,
            &self.draw_settings,
            &self.timings,
            resources,
            &mut self.mutable_resources,
            &mut self.mutable_sub_resources,
        );
    }
}

fn update_sonar(
    sonar: &mut Sonar,
    navigation: &Navigation,
    sub_size: (usize, usize),
    rock_grid: &RockGrid,
    mutable_resources: &mut MutableSubResources,
) {
    let center_x = (navigation.position.0 / 16 / 16) as usize;
    let center_y = (navigation.position.1 / 16 / 16) as usize;

    let sub_center_x = center_x + sub_size.0 / 2 / 16;
    let sub_center_y = center_y + sub_size.1 / 2 / 16;

    sonar.increase_pulse();

    if sonar.should_update() {
        find_visible_edge_cells(sonar, (sub_center_x, sub_center_y), rock_grid);
        mutable_resources.sonar_updated = true;
    }
}
