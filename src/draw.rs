use macroquad::prelude::{
    draw_line, draw_rectangle, draw_texture, draw_texture_ex, get_time, gl_use_default_material,
    gl_use_material, screen_height, screen_width, set_camera, vec2, vec3, Camera2D, Color,
    DrawTextureParams, Rect, Vec2, BLACK, DARKBLUE, GRAY, WHITE,
};

use crate::{
    objects::{Object, ObjectType},
    water::WaterGrid,
    wires::{WireColor, WireGrid},
    Resources,
};

#[derive(Debug, Default)]
pub(crate) struct Camera {
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: i32,
    pub dragging_from: (f32, f32),
    pub scrolling_from: f32,
    pub pointing_at: (usize, usize),
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
    water_grid: &WaterGrid,
    wire_grid: &WireGrid,
    camera: &Camera,
    draw_sea_water: bool,
    should_draw_objects: bool,
    draw_water_grid: bool,
    objects: &Vec<Object>,
    resources: &Resources,
    highlighting_object: &Option<(usize, bool)>,
) {
    set_camera(&camera.to_macroquad_camera());

    if draw_sea_water {
        draw_sea(&camera, resources);
    } else {
        draw_fake_sea();
    }

    let (width, height) = water_grid.size();

    draw_background(width, height, resources);

    draw_walls(water_grid, resources);

    draw_wires(wire_grid, resources);

    if should_draw_objects {
        draw_objects(objects, width, height, resources, highlighting_object);
    }

    if draw_water_grid {
        draw_water(water_grid);
    }
}

/*
fn draw_sea_caustics(resources: &Resources) {
    resources.sea_water.set_uniform("iTime", get_time() as f32);
    resources
        .sea_water
        .set_uniform("iResolution", vec2(0.3, 0.3));
    gl_use_material(resources.sea_water);
    draw_rect_at(vec2(0.0, 0.0), 500.0, DARKBLUE);
    gl_use_default_material();
}
*/

fn draw_sea(camera: &Camera, resources: &Resources) {
    let time_offset = vec2(0.1, 1.0) * get_time() as f32 * 0.03;
    let camera_offset = vec2(camera.offset_x, camera.offset_y) / 300.0;
    resources.sea_water.set_uniform("time_offset", time_offset);
    resources
        .sea_water
        .set_uniform("camera_offset", camera_offset);
    resources.sea_water.set_uniform("time", get_time() as f32);
    resources
        .sea_water
        .set_uniform("resolution", vec2(0.3, 0.3));
    resources
        .sea_water
        .set_texture("sea_dust", resources.sea_dust);
    gl_use_material(resources.sea_water);
    draw_rect_at(vec2(0.0, 0.0), 500.0, DARKBLUE);
    gl_use_default_material();
}

fn draw_fake_sea() {
    draw_rect_at(vec2(0.0, 0.0), 500.0, DARKBLUE);
}

fn draw_background(width: usize, height: usize, resources: &Resources) {
    let top_left = to_screen_coords(0, 0, width, height) - vec2(0.5, 0.5);
    draw_texture(resources.sub_background, top_left.x, top_left.y, WHITE);
}

fn draw_walls(grid: &WaterGrid, resources: &Resources) {
    let (width, height) = grid.size();

    for i in 0..width {
        for j in 0..height {
            let cell = grid.cell(i, j);

            if cell.is_wall() {
                let pos = to_screen_coords(i, j, width, height);
                draw_texture_ex(
                    resources.wall,
                    pos.x - 0.5,
                    pos.y - 0.5,
                    GRAY,
                    DrawTextureParams {
                        dest_size: Some(vec2(1.0, 1.0)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}

fn draw_water(grid: &WaterGrid) {
    let (width, height) = grid.size();

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

            let transparent_blue = Color::new(0.40, 0.75, 1.00, 0.75);

            if cell.is_wall() {
                // Drawn in draw_walls()
            } else if level > 0.0 && !cell.is_sea() {
                draw_rect_at(pos, size * level, transparent_blue);
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

fn draw_wires(grid: &WireGrid, resources: &Resources) {
    let (width, height) = grid.size();

    for x in 0..width {
        for y in 0..height {
            let pos = to_screen_coords(x, y, width, height);

            let cell = grid.cell(x, y);

            let colors = &[
                (WireColor::Orange, vec3(1.0, 1.0, 0.0)),
                (WireColor::Brown, vec3(0.0, 1.0, 1.0)),
                (WireColor::Blue, vec3(0.0, 0.0, 1.0)),
                (WireColor::Green, vec3(0.0, 1.0, 0.0)),
            ];

            for (wire_color, color) in colors {
                let value = match cell.value(*wire_color) {
                    crate::wires::WireValue::NotConnected => continue,
                    crate::wires::WireValue::NoSignal => 0,
                    crate::wires::WireValue::Power { value, .. } => *value.min(&100),
                    crate::wires::WireValue::Logic { value, .. } => {
                        value.clamp(&-100, &100).abs() as u8
                    }
                };

                let has_neighbours = grid.has_neighbours(*wire_color, x, y);

                let total_frames = 7;
                let wire_frame = match has_neighbours {
                    // [down, right, up, left]
                    [true, false, true, false] => 0,
                    [false, false, true, false] => 0,
                    [true, false, false, false] => 0,
                    [false, true, false, true] => 1,
                    [false, false, false, true] => 1,
                    [false, true, false, false] => 1,
                    [true, true, false, false] => 2,
                    [true, false, false, true] => 3,
                    [false, false, true, true] => 4,
                    [false, true, true, false] => 5,
                    _ => 6,
                };

                let _value = (value + 128) as f32 / u8::MAX as f32;
                let signal = cell.value(*wire_color).signal() as f32 / 256.0;

                resources.wire_material.set_uniform("wire_color", *color);
                resources.wire_material.set_uniform("signal", signal);
                resources
                    .wire_material
                    .set_texture("wires_texture", resources.wires);

                gl_use_material(resources.wire_material);

                // Textures are vertically split into equally-sized animation frames
                let frame_width = resources.wires.width();
                let frame_height = (resources.wires.height() as u16 / total_frames) as f32;
                let frame_x = 0.0;
                let frame_y = (frame_height as u16 * wire_frame) as f32;

                draw_texture_ex(
                    resources.wires,
                    pos.x - 0.5,
                    pos.y - 0.5,
                    BLACK,
                    DrawTextureParams {
                        dest_size: Some(vec2(1.0, 1.0)),
                        source: Some(Rect::new(frame_x, frame_y, frame_width, frame_height)),
                        ..Default::default()
                    },
                );

                gl_use_default_material();
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
            ObjectType::Reactor { .. } => resources.reactor,
            ObjectType::Lamp => resources.lamp,
            ObjectType::Gauge { .. } => resources.gauge,
            ObjectType::LargePump { .. } => resources.large_pump,
            ObjectType::JunctionBox => resources.junction_box,
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
                resources.hover_highlight.set_uniform("frame_y", frame_y);
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
