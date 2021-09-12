use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::{
        load_material, render_target, FilterMode, Image, ImageFormat, Material, MaterialParams,
        PipelineParams, RenderTarget, Texture2D, UniformType,
    },
};

use crate::{
    draw::Camera,
    game_state::{
        state::GameState,
        update::{SubmarineUpdatedEvent, UpdateEvent},
    },
    saveload::pixels_to_image,
    shadows::Edge,
};

pub(crate) struct Resources {
    pub settings: Texture2D,
    pub sea_water: Material,
    pub hover_highlight: Material,
    pub wire_material: Material,
    pub wall_material: Material,
    pub rock_material: Material,
    pub sonar_material: Material,
    pub shadow_material: Material,
    pub pointlight_material: Material,
    pub wires: Texture2D,
    pub sea_dust: Texture2D,
    pub wall: Texture2D,
    pub glass: Texture2D,
    pub rocks: Texture2D,
    pub hatch: Texture2D,
    pub door: Texture2D,
    pub reactor: Texture2D,
    pub lamp: Texture2D,
    pub gauge: Texture2D,
    pub small_pump: Texture2D,
    pub large_pump: Texture2D,
    pub junction_box: Texture2D,
    pub nav_controller: Texture2D,
    pub sonar: Texture2D,
    pub engine: Texture2D,
    pub turbulence: Texture2D,
    pub battery: Texture2D,
    pub bundle_input: Texture2D,
    pub bundle_output: Texture2D,
    pub docking_connector_top: Texture2D,
    pub docking_connector_bottom: Texture2D,
}

pub(crate) struct MutableResources {
    pub sea_rocks: Texture2D,
    pub sea_rocks_updated: bool,
    pub shadows: RenderTarget,
    pub screen: Texture2D,
}

pub(crate) struct MutableSubResources {
    pub sub_background_image: Image,
    pub sub_background: Texture2D,
    pub sub_walls: Texture2D,
    pub walls_updated: bool,
    pub sub_wires: RenderTarget,
    pub wires_updated: bool,
    pub sub_signals_image: Image,
    pub sub_signals: Texture2D,
    pub signals_updated: bool,
    pub new_sonar_target: RenderTarget,
    pub old_sonar_target: RenderTarget,
    pub sonar_updated: bool,
    pub sonar_cursor: Option<(usize, (f32, f32))>,
    pub turbulence_particles: Vec<TurbulenceParticle>,
    pub highlighting_object: Option<usize>,
    pub sub_cursor: (f32, f32),
    pub shadow_edges: Vec<Edge>,
    pub shadow_edges_updated: bool,
}

pub(crate) struct TurbulenceParticle {
    pub position: (f32, f32),
    pub frame: u8,
    pub speed: f32,
    pub life: u8,
}

impl Resources {
    pub fn new() -> Self {
        let sea_water = load_material(
            include_str!("vertex.glsl"),
            include_str!("water.glsl"),
            MaterialParams {
                uniforms: vec![
                    ("enable_dust".to_string(), UniformType::Float1),
                    ("enable_caustics".to_string(), UniformType::Float1),
                    ("time_offset".to_string(), UniformType::Float2),
                    ("camera_offset".to_string(), UniformType::Float2),
                    ("time".to_string(), UniformType::Float1),
                    ("world_size".to_string(), UniformType::Float2),
                    ("sea_dust_size".to_string(), UniformType::Float2),
                ],
                textures: vec!["sea_dust".to_string()],
                ..Default::default()
            },
        )
        .expect("Could not load material");

        fn load_texture(bytes: &[u8]) -> Texture2D {
            let texture = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
            texture.set_filter(FilterMode::Nearest);
            texture
        }

        let settings = load_texture(include_bytes!("../resources/settings.png"));
        let sea_dust = load_texture(include_bytes!("../resources/seadust.png"));
        let wires = load_texture(include_bytes!("../resources/wires.png"));
        let wall = load_texture(include_bytes!("../resources/wall.png"));
        let glass = load_texture(include_bytes!("../resources/glass.png"));
        let rocks = load_texture(include_bytes!("../resources/rocks.png"));
        let hatch = load_texture(include_bytes!("../resources/hatch.png"));
        let door = load_texture(include_bytes!("../resources/door.png"));
        let reactor = load_texture(include_bytes!("../resources/reactor.png"));
        let lamp = load_texture(include_bytes!("../resources/lamp.png"));
        let gauge = load_texture(include_bytes!("../resources/gauge.png"));
        let small_pump = load_texture(include_bytes!("../resources/smallpump.png"));
        let large_pump = load_texture(include_bytes!("../resources/largepump.png"));
        let junction_box = load_texture(include_bytes!("../resources/junctionbox.png"));
        let nav_controller = load_texture(include_bytes!("../resources/navcontroller.png"));
        let sonar = load_texture(include_bytes!("../resources/sonar.png"));
        let engine = load_texture(include_bytes!("../resources/engine.png"));
        let turbulence = load_texture(include_bytes!("../resources/turbulence.png"));
        let battery = load_texture(include_bytes!("../resources/battery.png"));
        let bundle_input = load_texture(include_bytes!("../resources/bundle_input.png"));
        let bundle_output = load_texture(include_bytes!("../resources/bundle_output.png"));
        let docking_connector_top =
            load_texture(include_bytes!("../resources/docking_connector_top.png"));
        let docking_connector_bottom =
            load_texture(include_bytes!("../resources/docking_connector_bottom.png"));

        sea_dust.set_filter(FilterMode::Linear);

        let blend_alpha = PipelineParams {
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            alpha_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Zero,
                BlendFactor::One,
            )),
            ..Default::default()
        };

        let hover_highlight = load_material(
            include_str!("vertex.glsl"),
            include_str!("highlight.glsl"),
            MaterialParams {
                uniforms: vec![
                    ("input_resolution".to_string(), UniformType::Float2),
                    ("frame_y".to_string(), UniformType::Float1),
                    ("frame_x".to_string(), UniformType::Float1),
                    ("frame_height".to_string(), UniformType::Float1),
                    ("frame_width".to_string(), UniformType::Float1),
                    ("clicked".to_string(), UniformType::Float1),
                ],
                textures: vec!["input_texture".to_string()],
                pipeline_params: blend_alpha,
            },
        )
        .expect("Could not load door highlight material");

        let wire_material = load_material(
            include_str!("vertex.glsl"),
            include_str!("wires.glsl"),
            MaterialParams {
                uniforms: vec![("grid_size".to_string(), UniformType::Float2)],
                textures: vec!["sub_wires".to_string(), "sub_signals".to_string()],
                pipeline_params: blend_alpha,
            },
        )
        .expect("Could not load wire material");

        let wall_material = load_material(
            include_str!("vertex.glsl"),
            include_str!("walls.glsl"),
            MaterialParams {
                uniforms: vec![("walls_size".to_string(), UniformType::Float2)],
                textures: vec![
                    "wall_texture".to_string(),
                    "glass_texture".to_string(),
                    "walls".to_string(),
                ],
                pipeline_params: blend_alpha,
            },
        )
        .expect("Could not load wall material");

        let rock_material = load_material(
            include_str!("vertex.glsl"),
            include_str!("rocks.glsl"),
            MaterialParams {
                uniforms: vec![("sea_rocks_size".to_string(), UniformType::Float2)],
                textures: vec!["rocks_texture".to_string(), "sea_rocks".to_string()],
                pipeline_params: blend_alpha,
            },
        )
        .expect("Could not load rock material");

        let sonar_material = load_material(
            include_str!("vertex.glsl"),
            include_str!("sonar.glsl"),
            MaterialParams {
                uniforms: vec![
                    ("sonar_texture_size".to_string(), UniformType::Float2),
                    ("pulse".to_string(), UniformType::Float1),
                ],
                textures: vec![
                    "new_sonar_texture".to_string(),
                    "old_sonar_texture".to_string(),
                ],
                pipeline_params: blend_alpha,
            },
        )
        .expect("Could not load sonar material");

        let shadow_material = load_material(
            include_str!("vertex.glsl"),
            include_str!("shadows.glsl"),
            MaterialParams {
                uniforms: vec![],
                textures: vec!["screen".to_string(), "shadows".to_string()],
                pipeline_params: blend_alpha,
            },
        )
        .expect("Could not load shadow material");

        let pointlight_material = load_material(
            include_str!("vertex.glsl"),
            include_str!("pointlight.glsl"),
            MaterialParams {
                uniforms: vec![
                    ("pointlight_size".to_string(), UniformType::Float2),
                    ("pointlight_position".to_string(), UniformType::Float2),
                ],
                textures: vec![],
                pipeline_params: blend_alpha,
            },
        )
        .expect("Could not load point light material");

        Resources {
            settings,
            sea_water,
            hover_highlight,
            wire_material,
            wall_material,
            rock_material,
            sonar_material,
            shadow_material,
            pointlight_material,
            wires,
            sea_dust,
            wall,
            glass,
            rocks,
            hatch,
            door,
            reactor,
            lamp,
            gauge,
            small_pump,
            large_pump,
            junction_box,
            nav_controller,
            sonar,
            engine,
            turbulence,
            battery,
            bundle_input,
            bundle_output,
            docking_connector_top,
            docking_connector_bottom,
        }
    }
}

impl Default for MutableResources {
    fn default() -> Self {
        Self::new()
    }
}

impl MutableResources {
    pub fn new() -> Self {
        MutableResources {
            sea_rocks: Texture2D::empty(),
            sea_rocks_updated: false,
            shadows: render_target(0, 0),
            screen: Texture2D::empty(),
        }
    }
}

impl MutableSubResources {
    pub fn new(sub_background_image: Image) -> Self {
        let sub_background = Texture2D::from_image(&sub_background_image);
        sub_background.set_filter(FilterMode::Nearest);

        MutableSubResources {
            sub_background_image,
            sub_background,
            sub_walls: Texture2D::empty(),
            walls_updated: true,
            sub_wires: render_target(0, 0),
            wires_updated: true,
            sub_signals_image: Image::empty(),
            sub_signals: Texture2D::empty(),
            signals_updated: true,
            new_sonar_target: render_target(0, 0),
            old_sonar_target: render_target(0, 0),
            sonar_updated: true,
            sonar_cursor: None,
            turbulence_particles: Vec::new(),
            highlighting_object: None,
            sub_cursor: (0.0, 0.0),
            shadow_edges: Vec::new(),
            shadow_edges_updated: true,
        }
    }
}

pub(crate) fn update_resources_from_events(
    events: impl Iterator<Item = UpdateEvent>,
    game_state: &GameState,
    mutable_sub_resources: &mut Vec<MutableSubResources>,
    camera: &mut Camera,
    current_submarine: &mut usize,
) {
    for event in events {
        match event {
            UpdateEvent::Submarine {
                submarine_id,
                submarine_event,
            } => {
                let mutable_sub_resources = mutable_sub_resources
                    .get_mut(submarine_id)
                    .expect("All submarines should have their own MutableSubResources instance");

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
                let submarine = game_state
                    .submarines
                    .last()
                    .expect("Submarine just created");
                let (width, height) = submarine.water_grid.size();
                let image = pixels_to_image(width, height, &submarine.background_pixels);
                mutable_sub_resources.push(MutableSubResources::new(image));

                // Change camera to its middle and set it as current
                *current_submarine = game_state.submarines.len() - 1;
                camera.offset_x = -(width as f32) / 2.0;
                camera.offset_y = -(height as f32) / 2.0;
            }
            UpdateEvent::GameStateReset => {
                // FIXME: Delete textures
                mutable_sub_resources.clear();

                // FIXME: factor out
                for submarine in &game_state.submarines {
                    let (width, height) = submarine.water_grid.size();
                    let image = pixels_to_image(width, height, &submarine.background_pixels);
                    mutable_sub_resources.push(MutableSubResources::new(image))
                }

                // Get last submarine
                let submarine = game_state
                    .submarines
                    .last()
                    .expect("Submarine just created");
                let (width, height) = submarine.water_grid.size();

                // Change camera to its middle and set it as current
                *current_submarine = game_state.submarines.len() - 1;
                camera.offset_x = -(width as f32) / 2.0;
                camera.offset_y = -(height as f32) / 2.0;
            }
        }
    }
}
