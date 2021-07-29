use macroquad::{miniquad::{BlendFactor, BlendState, BlendValue, Equation}, prelude::{BLACK, Camera2D, Color, DARKBLUE, DrawTextureParams, FilterMode, GRAY, ImageFormat, Material, MaterialParams, PipelineParams, Rect, SKYBLUE, Texture2D, UniformType, Vec2, WHITE, draw_line, draw_rectangle, draw_texture, draw_texture_ex, get_time, gl_use_default_material, gl_use_material, load_material, screen_height, screen_width, set_camera, vec2}};

use crate::{
    objects::{Object, ObjectType},
    water::WaterGrid,
};

#[derive(Debug, Default)]
pub(crate) struct Camera {
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: i32,
    pub dragging_from: (f32, f32),
    pub scrolling_from: f32,
}

pub struct ResourcesBuilder {
    sub_background: Option<Texture2D>,
}

pub struct Resources {
    sub_background: Texture2D,
    wall: Texture2D,
    door: Texture2D,
    vertical_door: Texture2D,
    hover_highlight: Material,
    sea_water: Material,
}

impl Camera {
    pub fn to_macroquad_camera(&self) -> Camera2D {
        let zoom = if screen_height() < screen_width() {
            vec2(screen_height() / screen_width(), -1.0) * 1.3
        } else {
            vec2(1.0, -screen_width() / screen_height())
        };

        let offset = vec2(-self.offset_x as f32 * 2.0, -self.offset_y as f32 * 2.0);

        Camera2D {
            zoom: zoom * (1.5 / 50.0) * self.user_zoom(),
            target: offset,
            ..Default::default()
        }
    }

    fn user_zoom(&self) -> f32 {
        1.0 / (1.0 - self.zoom as f32 / 64.0)
    }
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
                    ("iTime".to_string(), UniformType::Float1),
                    ("iResolution".to_string(), UniformType::Float2),
                ],
                ..Default::default()
            },
        )
        .expect("Could not load material");

        let wall = Texture2D::from_file_with_format(
            include_bytes!("../resources/wall.png"),
            Some(ImageFormat::Png),
        );
        wall.set_filter(FilterMode::Nearest);

        let door = Texture2D::from_file_with_format(
            include_bytes!("../resources/door.png"),
            Some(ImageFormat::Png),
        );
        door.set_filter(FilterMode::Nearest);

        let vertical_door = Texture2D::from_file_with_format(
            include_bytes!("../resources/vertical_door.png"),
            Some(ImageFormat::Png),
        );
        vertical_door.set_filter(FilterMode::Nearest);

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
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
                    ),
                    alpha_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Zero,
                        BlendFactor::One)
                    ),
                    ..Default::default()
                }
            },
        )
        .expect("Could not load door highlight material");

        Resources {
            sea_water,
            wall,
            sub_background: self.sub_background.expect("Sub Background not provided"),
            door,
            vertical_door,
            hover_highlight,
        }
    }
}

fn draw_rect_at(pos: Vec2, size: f32, color: Color) {
    draw_rectangle(pos.x - size, pos.y - size, size * 2.0, size * 2.0, color);
}

pub(crate) fn to_screen_coords(x: usize, y: usize, width: usize, height: usize) -> Vec2 {
    vec2(
        (x as i32 - (width as i32 / 2)) as f32,
        (y as i32 - (height as i32 / 2)) as f32,
    )
}

pub(crate) fn draw_game(
    grid: &WaterGrid,
    camera: &Camera,
    draw_sea_water: bool,
    should_draw_objects: bool,
    objects: &Vec<Object>,
    resources: &Resources,
    highlighting_object: &Option<(usize, bool)>,
) {
    let camera = camera.to_macroquad_camera();
    set_camera(&camera);

    if draw_sea_water {
        draw_sea(resources);
    } else {
        draw_fake_sea();
    }

    let (width, height) = grid.size();

    draw_water(grid, resources);

    if should_draw_objects {
        draw_objects(objects, width, height, resources, highlighting_object);
    }
}

fn draw_sea(resources: &Resources) {
    resources.sea_water.set_uniform("iTime", get_time() as f32);
    resources
        .sea_water
        .set_uniform("iResolution", vec2(0.3, 0.3));
    gl_use_material(resources.sea_water);
    draw_rect_at(vec2(0.0, 0.0), 500.0, DARKBLUE);
    gl_use_default_material();
}

fn draw_fake_sea() {
    draw_rect_at(vec2(0.0, 0.0), 500.0, DARKBLUE);
}

fn draw_water(grid: &WaterGrid, resources: &Resources) {
    let (width, height) = grid.size();

    let top_left = to_screen_coords(0, 0, width, height) - vec2(0.5, 0.5);
    draw_texture(resources.sub_background, top_left.x, top_left.y, WHITE);

    for i in 0..width {
        for j in 0..height {
            let pos = to_screen_coords(i, j, width, height);
            let level = grid.cell(i, j).amount_filled();
            let overlevel = grid.cell(i, j).amount_overfilled();
            let velocity = grid.cell(i, j).velocity();

            let level = if level != 0.0 && level < 0.5 {
                0.5
            } else {
                level
            };

            let size = 0.5;

            let cell = grid.cell(i, j);

            if cell.is_wall() {
                draw_texture_ex(resources.wall, pos.x - 0.5, pos.y - 0.5, GRAY, DrawTextureParams {
                    dest_size: Some(vec2(1.0, 1.0)),
                    ..Default::default()
                });
                //draw_rect_at(pos, size, GRAY);
            } else if level > 0.0 && !cell.is_sea() {
                draw_rect_at(pos, size * level, SKYBLUE);
                draw_rect_at(pos, size * overlevel, DARKBLUE);
            }

            if !cell.is_sea() && !cell.is_wall() && level != 0.0 {
                let velocity = vec2(velocity.0, velocity.1).normalize_or_zero() * 0.35;

                if velocity != vec2(0.0, 0.0) {
                    draw_line(
                        pos.x,
                        pos.y,
                        pos.x + velocity.x,
                        pos.y + velocity.y,
                        0.1,
                        BLACK,
                    );
                }
            }
        }
    }
}

pub(crate) fn object_rect(object: &Object, width: usize, height: usize) -> Rect {
    let pos = to_screen_coords(
        object.position_x as usize,
        object.position_y as usize,
        width,
        height,
    );

    let size = object.size();
    let size = vec2(size.0 as f32, size.1 as f32);

    Rect::new(pos.x + 0.5, pos.y + 0.5, size.x, size.y)
}

fn draw_objects(
    objects: &Vec<Object>,
    width: usize,
    height: usize,
    resources: &Resources,
    highlighting_object: &Option<(usize, bool)>,
) {
    for (obj_id, object) in objects.iter().enumerate() {
        let draw_rect = object_rect(object, width, height);

        let texture = match object.object_type {
            ObjectType::Door { .. } => resources.door,
            ObjectType::VerticalDoor { .. } => resources.vertical_door,
        };

        // Textures are vertically split into equally-sized animation frames
        let frame_width = texture.width();
        let frame_height = (texture.height() as u16 / object.frames) as f32;
        let frame_x = 0.0;
        let frame_y = (frame_height as u16 * object.current_frame) as f32;

        draw_texture_ex(
            texture,
            draw_rect.x,
            draw_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(draw_rect.size()),
                source: Some(Rect::new(frame_x, frame_y, frame_width, frame_height)),
                ..Default::default()
            },
        );

        if let Some((highlighting_object, _clicked)) = highlighting_object {
            if *highlighting_object == obj_id {
                let texture_resolution = vec2(texture.width(), texture.height());
                resources
                    .hover_highlight
                    .set_uniform("input_resolution", texture_resolution);
                resources
                    .hover_highlight
                    .set_uniform("frame_y", frame_y);
                    resources
                    .hover_highlight
                    .set_uniform("frame_height", frame_height);
                resources
                    .hover_highlight
                    .set_texture("input_texture", texture);
                gl_use_material(resources.hover_highlight);
                let r = draw_rect;
                draw_rectangle(r.x, r.y, r.w, r.h, DARKBLUE);
                gl_use_default_material();
            }
        }
    }
}
