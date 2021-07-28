use macroquad::prelude::{
    is_key_down, is_key_pressed, is_key_released, is_mouse_button_down, is_mouse_button_pressed,
    is_mouse_button_released, mouse_position, mouse_wheel, KeyCode, MouseButton, Rect, Vec2,
};

use crate::{
    app::Tool,
    draw::{object_rect, to_screen_coords, Camera},
    objects::{interact_with_object, Object},
    water::WaterGrid,
};

pub(crate) fn handle_keyboard_input(camera: &mut Camera, current_tool: &mut Tool) {
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        camera.offset_x += 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        camera.offset_x -= 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        camera.offset_y += 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        camera.offset_y -= 1.0;
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

pub(crate) fn handle_pointer_input(
    grid: &mut WaterGrid,
    objects: &mut Vec<Object>,
    camera: &mut Camera,
    tool: &Tool,
    dragging_object: &mut bool,
) {
    let macroquad_camera = camera.to_macroquad_camera();

    if is_mouse_button_pressed(MouseButton::Right) {
        camera.dragging_from = mouse_position();
        if cfg!(not(target_arch = "wasm32")) {
            // TODO: Bugged; it makes the egui windows act weird
            // set_cursor_grab(true);
        }
    }

    if is_mouse_button_released(MouseButton::Right) {
        if cfg!(not(target_arch = "wasm32")) {
            // set_cursor_grab(false);
        }
    }

    if is_mouse_button_down(MouseButton::Right) {
        let new_position = mouse_position();

        let old = macroquad_camera.screen_to_world(Vec2::from(camera.dragging_from));
        let new = macroquad_camera.screen_to_world(Vec2::from(new_position));

        let delta = (new - old) * 0.5;

        camera.offset_x += delta.x;
        camera.offset_y += delta.y;

        camera.dragging_from = new_position;
    }

    let scroll = mouse_wheel().1;
    if scroll != 0.0 {
        let multiplier = if cfg!(target_arch = "wasm32") {
            0.1
        } else {
            1.0
        };

        camera.zoom = (camera.zoom + (scroll * multiplier) as i32 * 4).clamp(-512, 36);
    }

    let mouse_position = macroquad_camera.screen_to_world(mouse_position().into());

    let (width, height) = grid.size();

    if is_mouse_button_pressed(MouseButton::Left) {
        for object in objects {
            let draw_rect = object_rect(object, width, height);

            if draw_rect.contains(mouse_position) {
                interact_with_object(object);
                *dragging_object = true;
                return;
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        *dragging_object = false;
    }

    // Painting the grid
    if !is_mouse_button_down(MouseButton::Left) || *dragging_object {
        return;
    }

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
