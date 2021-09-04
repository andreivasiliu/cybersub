use std::collections::VecDeque;

use crate::{
    client::{connect, RemoteConnection},
    draw::{draw_game, Camera, DrawSettings},
    game_state::objects::ObjectType,
    game_state::state::GameState,
    game_state::wires::WireColor,
    game_state::{
        state::SubmarineTemplate,
        update::{update_game, Command, SubmarineUpdatedEvent, UpdateEvent},
    },
    input::{handle_keyboard_input, handle_pointer_input},
    resources::{MutableResources, MutableSubResources, Resources},
    saveload::{load_rocks_from_png, load_template_from_data, pixels_to_image, save_to_file_data},
    ui::{draw_ui, UiState},
    SubmarineFileData,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::server::{serve, LocalClient, Server};

pub struct CyberSubApp {
    pub timings: Timings,
    ui_state: UiState,
    game_state: GameState,
    game_settings: GameSettings,
    commands: Vec<Command>,
    update_events: Vec<UpdateEvent>,
    update_source: UpdateSource,
    resources: Resources,
    mutable_resources: MutableResources,
    mutable_sub_resources: Vec<MutableSubResources>,
}

pub(crate) struct GameSettings {
    pub draw_settings: DrawSettings,
    pub network_settings: NetworkSettings,
    pub camera: Camera,
    pub current_submarine: usize,
    pub current_tool: Tool,
    pub quit_game: bool,
    pub dragging_object: bool,
    pub highlighting_settings: bool,
    pub last_update: Option<f64>,
    pub last_draw: Option<f64>,
    pub animation_ticks: u32,
    pub placing_object: Option<PlacingObject>,
    pub submarine_templates: Vec<(String, SubmarineTemplate)>,
}

pub(crate) struct NetworkSettings {
    pub server_tcp_address: String,
    pub server_ws_address: String,
    pub client_tcp_address: String,
    pub client_ws_address: String,
    pub start_server: bool,
    pub server_started: bool,
    pub connect_client: bool,
    pub client_connected: bool,
    pub network_status: String,
    pub network_error: Option<String>,
    pub download_progress: Option<u8>,
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
    pub fps_average: u32,
    pub frame_time: u32,
    pub fps_history: VecDeque<(f64, f64)>,
    pub fps_average_history: VecDeque<(f64, f64)>,
}

pub(crate) struct PlacingObject {
    pub submarine: usize,
    pub position: Option<(usize, usize)>,
    pub object_type: ObjectType,
}

enum UpdateSource {
    Local,
    #[cfg(not(target_arch = "wasm32"))]
    LocalServer(Server, LocalClient),
    Remote(RemoteConnection),
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
            draw_shadows: true,
            debug_shadows: false,
        };

        let network_settings = NetworkSettings {
            server_tcp_address: "127.0.0.1:3300".to_string(),
            server_ws_address: "0.0.0.0:3380".to_string(),
            client_tcp_address: "127.0.0.1:3300".to_string(),
            client_ws_address: "ws://192.168.15.101:3380".to_string(),
            start_server: false,
            server_started: false,
            connect_client: false,
            client_connected: false,
            network_status: "Not connected".to_string(),
            network_error: None,
            download_progress: None,
        };

        Self {
            timings: Timings::default(),
            game_settings: GameSettings {
                draw_settings,
                network_settings,
                camera: Camera {
                    zoom: -200,
                    ..Default::default()
                },
                current_submarine: 0,
                current_tool: Tool::Interact,
                quit_game: false,
                dragging_object: false,
                highlighting_settings: false,
                last_update: None,
                last_draw: None,
                animation_ticks: 0,
                placing_object: None,
                submarine_templates: Vec::new(),
            },
            commands: Vec::new(),
            update_events: Vec::new(),
            update_source: UpdateSource::Local,
            game_state: GameState::default(),
            ui_state: UiState::default(),
            resources: Resources::new(),
            mutable_resources: MutableResources::new(),
            mutable_sub_resources: Vec::new(),
        }
    }
}

impl CyberSubApp {
    pub fn load_submarine_template(
        &mut self,
        name: impl Into<String>,
        file_data: SubmarineFileData,
    ) -> Result<usize, String> {
        let template = load_template_from_data(file_data)?;
        self.game_settings
            .submarine_templates
            .push((name.into(), template));
        Ok(self.game_settings.submarine_templates.len() - 1)
    }

    pub fn add_submarine(&mut self, template_index: usize) {
        let (_name, template) = self
            .game_settings
            .submarine_templates
            .get(template_index)
            .expect("Template was requested this frame")
            .clone();

        let (width, height) = template.size;

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

        self.commands.push(Command::CreateSubmarine {
            submarine_template: Box::new(template),
            rock_position: (pos_x as usize, pos_y as usize),
        });
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

    pub fn start_server(&mut self) {
        self.game_settings.network_settings.start_server = true;
    }

    pub fn join_server(&mut self) {
        self.game_settings.network_settings.connect_client = true;
    }

    pub fn load_rocks(&mut self, world_bytes: &[u8]) {
        self.game_state.rock_grid = load_rocks_from_png(world_bytes);
    }

    pub fn update_game(&mut self, game_time: f64) {
        self.game_settings.animation_ticks = 0;

        if let Some(last_draw) = self.game_settings.last_draw {
            let mut delta = (game_time - last_draw).clamp(0.0, 0.5);

            while delta > 0.0 {
                // 60 animation updates per second, regardless of FPS

                delta -= 1.0 / 60.0;
                self.game_settings.animation_ticks += 1;
            }
        }

        let last_update = self.game_settings.last_update.get_or_insert(game_time);

        while *last_update < game_time {
            // 60 updates per second, regardless of FPS
            *last_update += 1.0 / 60.0;

            if true {
                let commands = self.commands.drain(0..self.commands.len());
                self.update_source.update(
                    &mut self.game_state,
                    commands,
                    &mut self.update_events,
                    &mut self.game_settings.network_settings,
                );

                for event in self.update_events.drain(0..self.update_events.len()) {
                    match event {
                        UpdateEvent::Submarine {
                            submarine_id,
                            submarine_event,
                        } => {
                            let mutable_sub_resources = self.mutable_sub_resources.get_mut(submarine_id).expect("All submarines should have their own MutableSubResources instance");

                            match submarine_event {
                                SubmarineUpdatedEvent::Sonar => {
                                    mutable_sub_resources.sonar_updated = true;
                                }
                                SubmarineUpdatedEvent::Walls => {
                                    mutable_sub_resources.walls_updated = true;
                                    mutable_sub_resources.shadow_edges_updated = true;
                                }
                                SubmarineUpdatedEvent::Wires => {
                                    mutable_sub_resources.wires_updated = true;
                                }
                                SubmarineUpdatedEvent::Signals => {
                                    mutable_sub_resources.signals_updated = true;
                                }
                            }
                        }
                        UpdateEvent::SubmarineCreated => {
                            let submarine = self
                                .game_state
                                .submarines
                                .last()
                                .expect("Submarine just created");
                            let (width, height) = submarine.water_grid.size();
                            let image =
                                pixels_to_image(width, height, &submarine.background_pixels);
                            self.mutable_sub_resources
                                .push(MutableSubResources::new(image));

                            // Change camera to its middle and set it as current
                            self.game_settings.current_submarine =
                                self.game_state.submarines.len() - 1;
                            self.game_settings.camera.offset_x = -(width as f32) / 2.0;
                            self.game_settings.camera.offset_y = -(height as f32) / 2.0;
                        }
                        UpdateEvent::GameStateReset => {
                            // FIXME: Delete textures
                            self.mutable_sub_resources.clear();

                            // FIXME: factor out
                            for submarine in &self.game_state.submarines {
                                let (width, height) = submarine.water_grid.size();
                                let image =
                                    pixels_to_image(width, height, &submarine.background_pixels);
                                self.mutable_sub_resources
                                    .push(MutableSubResources::new(image))
                            }

                            // Get last submarine
                            let submarine = self
                                .game_state
                                .submarines
                                .last()
                                .expect("Submarine just created");
                            let (width, height) = submarine.water_grid.size();

                            // Change camera to its middle and set it as current
                            self.game_settings.current_submarine =
                                self.game_state.submarines.len() - 1;
                            self.game_settings.camera.offset_x = -(width as f32) / 2.0;
                            self.game_settings.camera.offset_y = -(height as f32) / 2.0;
                        }
                    }
                }
            }
        }

        self.game_settings.last_draw = Some(game_time);

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

impl UpdateSource {
    fn update(
        &mut self,
        game_state: &mut GameState,
        commands: impl Iterator<Item = Command>,
        events: &mut Vec<UpdateEvent>,
        network_settings: &mut NetworkSettings,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        if network_settings.start_server {
            assert!(!network_settings.client_connected);

            let (server, client) = serve(
                network_settings.server_tcp_address.clone(),
                network_settings.server_ws_address.clone(),
            );

            *self = UpdateSource::LocalServer(server, client);

            network_settings.start_server = false;
            network_settings.server_started = true;
            network_settings.network_status = format!(
                "Listening on tcp://{} and ws://{}",
                network_settings.server_tcp_address, network_settings.server_ws_address,
            );
            network_settings.network_error = None;
        }

        if network_settings.connect_client {
            assert!(!network_settings.server_started);

            let address = if cfg!(target_arch = "wasm32") {
                &network_settings.client_ws_address
            } else {
                &network_settings.client_tcp_address
            };

            match connect(address) {
                Ok(remote_connection) => {
                    *self = UpdateSource::Remote(remote_connection);
                    network_settings.client_connected = true;
                    network_settings.network_status = format!("Connected to {}", address);
                    network_settings.network_error = None;
                }
                Err(error) => {
                    network_settings.network_error = Some(error);
                }
            };

            network_settings.connect_client = false;
        }

        match self {
            UpdateSource::Local => {
                update_game(commands, game_state, events);
            }
            #[cfg(not(target_arch = "wasm32"))]
            UpdateSource::LocalServer(server, local_client) => {
                local_client.send_commands(commands);
                server.relay_messages();
                server.tick(game_state, events);
            }
            UpdateSource::Remote(remote_connection) => {
                match remote_connection.send_messages(commands) {
                    Ok(()) => {
                        remote_connection.receive_messages(&mut network_settings.download_progress);
                    }
                    Err(err) => {
                        network_settings.network_error = Some(err);
                    }
                }

                while let Some(commands) = remote_connection.receive_commands(game_state, events) {
                    update_game(commands, game_state, events);
                }
            }
        }
    }
}
