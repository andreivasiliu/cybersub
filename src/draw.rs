use macroquad::prelude::{
    draw_line, draw_rectangle, draw_texture, get_time, gl_use_default_material, gl_use_material,
    load_material, screen_height, screen_width, set_camera, vec2, Camera2D, Color, ImageFormat,
    Material, MaterialParams, Texture2D, UniformType, Vec2, BLACK, DARKBLUE, GRAY, SKYBLUE, WHITE,
};

use crate::water::WaterGrid;

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
        Resources {
            sea_water,
            sub_background: self.sub_background.expect("Sub Background not provided"),
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

pub(crate) fn draw_game(grid: &WaterGrid, camera: &Camera, resources: &Resources) {
    let camera = camera.to_macroquad_camera();
    set_camera(&camera);

    let (width, height) = grid.size();

    resources.sea_water.set_uniform("iTime", get_time() as f32);
    resources
        .sea_water
        .set_uniform("iResolution", vec2(0.3, 0.3));

    gl_use_material(resources.sea_water);
    draw_rect_at(vec2(0.0, 0.0), 500.0, BLACK);

    gl_use_default_material();
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

            // draw_rect_at(pos, size * 1.05, GRAY);
            // draw_rect_at(pos, size, BLACK);

            let cell = grid.cell(i, j);

            if cell.is_wall() {
                draw_rect_at(pos, size, GRAY);
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
