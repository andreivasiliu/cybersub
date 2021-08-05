use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::{
        load_material, render_target, FilterMode, Image, ImageFormat, Material, MaterialParams,
        PipelineParams, RenderTarget, Texture2D, UniformType,
    },
};

pub struct Resources {
    pub(crate) sea_water: Material,
    pub(crate) hover_highlight: Material,
    pub(crate) wire_material: Material,
    pub(crate) wall_material: Material,
    pub(crate) rock_material: Material,
    pub(crate) sonar_material: Material,
    pub(crate) wires: Texture2D,
    pub(crate) sub_background: Texture2D,
    pub(crate) sea_dust: Texture2D,
    pub(crate) wall: Texture2D,
    pub(crate) rocks: Texture2D,
    pub(crate) door: Texture2D,
    pub(crate) vertical_door: Texture2D,
    pub(crate) reactor: Texture2D,
    pub(crate) lamp: Texture2D,
    pub(crate) gauge: Texture2D,
    pub(crate) large_pump: Texture2D,
    pub(crate) junction_box: Texture2D,
    pub(crate) nav_controller: Texture2D,
    pub(crate) sonar: Texture2D,
}

pub struct ResourcesBuilder {
    sub_background: Option<Texture2D>,
}

pub struct MutableResources {
    pub(crate) sea_rocks: Texture2D,
    pub(crate) sea_rocks_updated: bool,
}

pub struct MutableSubResources {
    pub(crate) sub_walls: Texture2D,
    pub(crate) walls_updated: bool,
    pub(crate) sub_wires: RenderTarget,
    pub(crate) wires_updated: bool,
    pub(crate) sub_signals_image: Image,
    pub(crate) sub_signals: Texture2D,
    pub(crate) signals_updated: bool,
    pub(crate) new_sonar_target: RenderTarget,
    pub(crate) old_sonar_target: RenderTarget,
    pub(crate) sonar_updated: bool,
}

impl ResourcesBuilder {
    pub fn new() -> Self {
        ResourcesBuilder {
            sub_background: None,
        }
    }

    pub fn sub_background(mut self, bytes: &[u8]) -> Self {
        self.sub_background = Some(Texture2D::from_file_with_format(
            bytes,
            Some(ImageFormat::Png),
        ));
        self.sub_background
            .as_mut()
            .unwrap()
            .set_filter(FilterMode::Nearest);
        self
    }

    pub fn build(self) -> Resources {
        let sea_water = load_material(
            include_str!("vertex.glsl"),
            include_str!("water.glsl"),
            MaterialParams {
                uniforms: vec![
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

        let sea_dust = load_texture(include_bytes!("../resources/seadust.png"));
        let wires = load_texture(include_bytes!("../resources/wires.png"));
        let wall = load_texture(include_bytes!("../resources/wall.png"));
        let rocks = load_texture(include_bytes!("../resources/rocks.png"));
        let door = load_texture(include_bytes!("../resources/door.png"));
        let vertical_door = load_texture(include_bytes!("../resources/vertical_door.png"));
        let reactor = load_texture(include_bytes!("../resources/reactor.png"));
        let lamp = load_texture(include_bytes!("../resources/lamp.png"));
        let gauge = load_texture(include_bytes!("../resources/gauge.png"));
        let large_pump = load_texture(include_bytes!("../resources/largepump.png"));
        let junction_box = load_texture(include_bytes!("../resources/junctionbox.png"));
        let nav_controller = load_texture(include_bytes!("../resources/navcontroller.png"));
        let sonar = load_texture(include_bytes!("../resources/sonar.png"));

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
                textures: vec!["wall_texture".to_string(), "walls".to_string()],
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
            sea_water,
            hover_highlight,
            wire_material,
            wall_material,
            rock_material,
            sonar_material,
            wires,
            sub_background: self.sub_background.expect("Sub Background not provided"),
            sea_dust,
            wall,
            rocks,
            door,
            vertical_door,
            reactor,
            lamp,
            gauge,
            large_pump,
            junction_box,
            nav_controller,
            sonar,
        }
    }
}

impl MutableResources {
    pub fn new() -> Self {
        MutableResources {
            sea_rocks: Texture2D::empty(),
            sea_rocks_updated: false,
        }
    }
}

impl MutableSubResources {
    pub fn new() -> Self {
        MutableSubResources {
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
        }
    }
}
