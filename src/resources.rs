use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::{
        load_material, render_target, FilterMode, Image, ImageFormat, Material, MaterialParams,
        PipelineParams, RenderTarget, Texture2D, UniformType,
    },
};

pub(crate) struct Resources {
    pub settings: Texture2D,
    pub sea_water: Material,
    pub hover_highlight: Material,
    pub wire_material: Material,
    pub wall_material: Material,
    pub rock_material: Material,
    pub sonar_material: Material,
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
}

pub(crate) struct MutableResources {
    pub sea_rocks: Texture2D,
    pub sea_rocks_updated: bool,
    pub collisions: Vec<(usize, usize)>,
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
    pub turbulence_particles: Vec<TurbulenceParticle>,
    pub collisions: Vec<(usize, usize)>,
    pub highlighting_object: Option<usize>,
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
                    ("frame_height".to_string(), UniformType::Float1),
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
                textures: vec!["wall_texture".to_string(), "glass_texture".to_string(), "walls".to_string()],
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

        Resources {
            settings,
            sea_water,
            hover_highlight,
            wire_material,
            wall_material,
            rock_material,
            sonar_material,
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
            collisions: Vec::new(),
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
            walls_updated: false,
            sub_wires: render_target(0, 0),
            wires_updated: false,
            sub_signals: Texture2D::empty(),
            sub_signals_image: Image::empty(),
            signals_updated: false,
            new_sonar_target: render_target(0, 0),
            old_sonar_target: render_target(0, 0),
            sonar_updated: false,
            turbulence_particles: Vec::new(),
            collisions: Vec::new(),
            highlighting_object: None,
        }
    }
}
