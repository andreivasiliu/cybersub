use macroquad::prelude::{
    draw_line, draw_rectangle, is_key_down, is_key_pressed, is_key_released, is_mouse_button_down,
    is_mouse_button_pressed, mouse_position, mouse_wheel, screen_height, screen_width, set_camera,
    vec2, Camera2D, Color, KeyCode, MouseButton, Rect, Vec2, BLACK, DARKBLUE, GRAY, SKYBLUE,
};

use crate::{app::Tool, water::WaterGrid};

#[derive(Debug, Default)]
pub(crate) struct Camera {
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: i32,
    pub dragging_from: (f32, f32),
    pub scrolling_from: f32,
}

impl Camera {
    pub fn to_macroquad_camera(&self) -> Camera2D {
        let zoom = if screen_height() < screen_width() {
            vec2(screen_height() / screen_width(), 1.0) * 1.3
        } else {
            vec2(1.0, screen_width() / screen_height())
        };

        let offset = vec2(-self.offset_x as f32 / 2.0, -self.offset_y as f32 / 2.0);

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

pub(crate) fn handle_keyboard_input(camera: &mut Camera, current_tool: &mut Tool) {
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        camera.offset_x += 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        camera.offset_x -= 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        camera.offset_y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        camera.offset_y += 1.0;
    }
    if is_key_down(KeyCode::KpAdd) {
        camera.zoom += 1;
    }
    if is_key_down(KeyCode::KpSubtract) {
        camera.zoom -= 1;
    }
    if is_key_pressed(KeyCode::LeftShift) {
        *current_tool = Tool::RemoveWalls;
    }
    if is_key_pressed(KeyCode::LeftControl) {
        *current_tool = Tool::AddWalls;
    }
    if is_key_released(KeyCode::LeftShift) || is_key_released(KeyCode::LeftControl) {
        *current_tool = Tool::AddWater;
    }
}

pub(crate) fn handle_pointer_input(grid: &mut WaterGrid, camera: &mut Camera, tool: &Tool) {
    let macroquad_camera = camera.to_macroquad_camera();

    if is_mouse_button_pressed(MouseButton::Right) {
        camera.dragging_from = mouse_position();
    }

    if is_mouse_button_down(MouseButton::Right) {
        let new_position = mouse_position();

        let old = macroquad_camera.screen_to_world(Vec2::from(camera.dragging_from));
        let new = macroquad_camera.screen_to_world(Vec2::from(new_position));

        let delta = (new - old) * 2.0;

        camera.offset_x += delta.x;
        camera.offset_y += delta.y;

        camera.dragging_from = new_position;
    }

    let scroll = mouse_wheel().1;
    if scroll != 0.0 {
        camera.zoom = (camera.zoom + scroll as i32 * 4).clamp(-512, 36);
    }

    if !is_mouse_button_down(MouseButton::Left) {
        return;
    }

    let mouse_position = macroquad_camera.screen_to_world(mouse_position().into());

    let (width, height) = grid.size();

    // Yes, this is silly. I'm just too lazy to figure out the math to find i/j directly.
    for i in 0..width {
        for j in 0..height {
            let pos = to_screen_coords(i, j, width, height);

            let size = 0.5;
            let rect = Rect::new(pos.x - size, pos.y - size, size * 2.0, size * 2.0);

            if rect.contains(mouse_position) {
                let cell = grid.cell_mut(i, j);
                match tool {
                    Tool::AddWater => cell.fill(),
                    Tool::AddWalls => cell.make_wall(),
                    Tool::RemoveWalls => cell.clear_wall(),
                }
                return;
            }
        }
    }
}

fn draw_rect_at(pos: Vec2, size: f32, color: Color) {
    draw_rectangle(pos.x - size, pos.y - size, size * 2.0, size * 2.0, color);
}

pub(crate) fn to_screen_coords(x: usize, y: usize, width: usize, height: usize) -> Vec2 {
    vec2(
        (x as i32 - (width as i32 / 2)) as f32,
        -((y as i32 - (height as i32 / 2)) as f32),
    )
}

pub(crate) fn draw_game(grid: &WaterGrid, camera: &Camera) {
    let camera = camera.to_macroquad_camera();
    set_camera(&camera);

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

            // draw_rect_at(pos, size * 1.05, GRAY);
            // draw_rect_at(pos, size, BLACK);

            if grid.cell(i, j).is_wall() {
                draw_rect_at(pos, size, GRAY);
            } else if level > 0.0 {
                draw_rect_at(pos, size * level, SKYBLUE);
                draw_rect_at(pos, size * overlevel, DARKBLUE);
            }

            let velocity = vec2(velocity.0, -velocity.1).normalize_or_zero() * 0.35;

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
