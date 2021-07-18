use macroquad::{
    camera::{set_camera, Camera2D},
    prelude::{
        draw_line, draw_rectangle, is_mouse_button_down, mouse_position, screen_height,
        screen_width, vec2, Color, MouseButton, Rect, Vec2, BLACK, DARKBLUE, GRAY, SKYBLUE,
    },
};

use crate::water::WaterGrid;

#[derive(Debug, Clone, Copy)]
enum Action {
    Fill,
    MakeWall,
    ClearWall,
}

pub(crate) fn handle_input(grid: &mut WaterGrid) {
    let action = if is_mouse_button_down(macroquad::prelude::MouseButton::Left) {
        Action::Fill
    } else if is_mouse_button_down(MouseButton::Right) {
        Action::MakeWall
    } else if is_mouse_button_down(MouseButton::Middle) {
        Action::ClearWall
    } else {
        return;
    };

    let mouse_position = camera().screen_to_world(mouse_position().into());

    let width = 60;
    let height = 40;

    // Yes, this is silly. I'm just too lazy to figure out the math to find i/j directly.
    for i in 0..width {
        for j in 0..height {
            let pos = to_screen_coords(i, j, width, height);

            let size = 0.007;
            let rect = Rect::new(pos.x - size, pos.y - size, size * 2.0, size * 2.0);

            if rect.contains(mouse_position) {
                let cell = grid.cell_mut(i, j);
                match action {
                    Action::Fill => cell.fill(),
                    Action::MakeWall => cell.make_wall(),
                    Action::ClearWall => cell.clear_wall(),
                }
                return;
            }
        }
    }
}

fn draw_rect_at(pos: Vec2, size: f32, color: Color) {
    draw_rectangle(pos.x - size, pos.y - size, size * 2.0, size * 2.0, color);
}

fn to_screen_coords(x: usize, y: usize, width: usize, height: usize) -> Vec2 {
    let view_size = 50;

    vec2(
        (x as i32 - (width as i32 / 2)) as f32 / view_size as f32 + 1.0 / view_size as f32 / 2.0,
        -((y as i32 - (height as i32 / 2)) as f32 / view_size as f32
            + 1.0 / view_size as f32 / 2.0),
    )
}

fn camera() -> Camera2D {
    let zoom = if screen_height() < screen_width() {
        vec2(screen_height() / screen_width(), 1.0) * 1.3
    } else {
        vec2(1.0, screen_width() / screen_height())
    };

    Camera2D {
        zoom: zoom * 1.5,
        ..Default::default()
    }
}

pub(crate) fn draw_game(grid: &WaterGrid) {
    let camera = camera();

    set_camera(&camera);

    let width = 60;
    let height = 40;

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

            let size = 0.007;

            draw_rect_at(pos, size * 1.05, GRAY);
            draw_rect_at(pos, size, BLACK);

            if grid.cell(i, j).is_wall() {
                draw_rect_at(pos, size, GRAY);
            } else {
                draw_rect_at(pos, size * level, SKYBLUE);
                draw_rect_at(pos, size * overlevel, DARKBLUE);
            }

            let velocity = vec2(velocity.0, -velocity.1).normalize_or_zero() * 0.007;

            draw_line(
                pos.x,
                pos.y,
                pos.x + velocity.x,
                pos.y + velocity.y,
                0.002,
                BLACK,
            );
        }
    }
}
