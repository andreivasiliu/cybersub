use macroquad::prelude::{
    is_key_down, is_key_pressed, is_key_released, is_mouse_button_down, is_mouse_button_pressed,
    is_mouse_button_released, mouse_position, mouse_wheel, KeyCode, MouseButton, Vec2,
};

use crate::{
    app::{SubmarineState, Tool},
    draw::{object_rect, Camera},
    objects::{hover_over_object, interact_with_object},
    resources::MutableSubResources,
    wires::WireColor,
};

fn from_screen_coords(pos: Vec2, width: usize, height: usize) -> (usize, usize) {
    (
        (pos.x + (width as f32 / 2.0)) as usize,
        (pos.y + (height as f32 / 2.0)) as usize,
    )
}

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
        *current_tool = Tool::RemoveWall;
    }
    if is_key_pressed(KeyCode::LeftControl) {
        *current_tool = Tool::AddWall;
    }
    if is_key_released(KeyCode::LeftShift) || is_key_released(KeyCode::LeftControl) {
        *current_tool = Tool::AddWater;
    }
}

pub(crate) fn handle_pointer_input(
    submarine: &mut SubmarineState,
    mutable_resources: &mut MutableSubResources,
    camera: &mut Camera,
    tool: &Tool,
    dragging_object: &mut bool,
    highlighting_object: &mut Option<(usize, bool)>,
) {
    // FIXME: use actual current submarine
    let macroquad_camera = camera.to_macroquad_camera(Some(submarine.navigation.position));
    let (width, height) = submarine.water_grid.size();

    camera.pointing_at = from_screen_coords(
        macroquad_camera.screen_to_world(mouse_position().into()),
        width,
        height,
    );

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

    *highlighting_object = None;

    for (obj_index, object) in submarine.objects.iter_mut().enumerate() {
        let draw_rect = object_rect(object, width, height);

        if draw_rect.contains(mouse_position) {
            *highlighting_object = Some((obj_index, false));

            let hover_position = mouse_position - draw_rect.point();

            hover_over_object(object, (hover_position.x, hover_position.y));

            if is_mouse_button_pressed(MouseButton::Left) {
                interact_with_object(object);
                *dragging_object = true;
                *highlighting_object = Some((obj_index, true));
                return;
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        *dragging_object = false;
    }

    // Painting the grid
    if is_mouse_button_down(MouseButton::Left) && !*dragging_object {
        let (x, y) = camera.pointing_at;

        if x < width && y < height {
            let water_cell = submarine.water_grid.cell_mut(x, y);
            let wire_cell = submarine.wire_grid.cell_mut(x, y);

            match tool {
                Tool::AddWater => water_cell.fill(),
                Tool::AddWall => water_cell.make_wall(),
                Tool::RemoveWall => water_cell.clear_wall(),
                Tool::AddBrownWire => wire_cell.make_wire(WireColor::Brown),
                Tool::AddPurpleWire => wire_cell.make_wire(WireColor::Purple),
                Tool::AddBlueWire => wire_cell.make_wire(WireColor::Blue),
                Tool::AddGreenWire => wire_cell.make_wire(WireColor::Green),
            }

            mutable_resources.walls_updated = true;
            mutable_resources.wires_updated = true;
        }
    }
}
