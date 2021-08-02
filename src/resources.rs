use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::{
        load_material, FilterMode, ImageFormat, Material, MaterialParams, PipelineParams,
        Texture2D, UniformType,
    },
};

pub struct Resources {
    pub(crate) sea_water: Material,
    pub(crate) hover_highlight: Material,
    pub(crate) wire_material: Material,
    pub(crate) wires: Texture2D,
    pub(crate) sub_background: Texture2D,
    pub(crate) sea_dust: Texture2D,
    pub(crate) wall: Texture2D,
    pub(crate) door: Texture2D,
    pub(crate) vertical_door: Texture2D,
    pub(crate) reactor: Texture2D,
    pub(crate) lamp: Texture2D,
    pub(crate) gauge: Texture2D,
    pub(crate) large_pump: Texture2D,
    pub(crate) junction_box: Texture2D,
}

pub struct ResourcesBuilder {
    sub_background: Option<Texture2D>,
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
                    ("resolution".to_string(), UniformType::Float2),
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
        let door = load_texture(include_bytes!("../resources/door.png"));
        let vertical_door = load_texture(include_bytes!("../resources/vertical_door.png"));
        let reactor = load_texture(include_bytes!("../resources/reactor.png"));
        let lamp = load_texture(include_bytes!("../resources/lamp.png"));
        let gauge = load_texture(include_bytes!("../resources/gauge.png"));
        let large_pump = load_texture(include_bytes!("../resources/largepump.png"));
        let junction_box = load_texture(include_bytes!("../resources/junctionbox.png"));

        sea_dust.set_filter(FilterMode::Linear);

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
                pipeline_params: PipelineParams {
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
                },
            },
        )
        .expect("Could not load door highlight material");

        let wire_material = load_material(
            include_str!("vertex.glsl"),
            include_str!("wires.glsl"),
            MaterialParams {
                uniforms: vec![
                    ("wire_color".to_string(), UniformType::Float3),
                    ("signal".to_string(), UniformType::Float1),
                ],
                textures: vec!["wires_texture".to_string()],
                pipeline_params: PipelineParams {
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
                },
            },
        )
        .expect("Could not load door highlight material");

        Resources {
            sea_water,
            hover_highlight,
            wire_material,
            wires,
            sub_background: self.sub_background.expect("Sub Background not provided"),
            sea_dust,
            wall,
            door,
            vertical_door,
            reactor,
            lamp,
            gauge,
            large_pump,
            junction_box,
        }
    }
}