use macroquad::prelude::{
    is_key_down, is_mouse_button_down, is_mouse_button_pressed, is_mouse_button_released,
    mouse_position, mouse_wheel, KeyCode, MouseButton, Rect, Vec2,
};

use crate::{
    app::{GameSettings, SubmarineState, Tool},
    draw::{object_rect, object_size, Camera},
    objects::hover_over_object,
    resources::MutableSubResources,
    update::{CellCommand, Command},
};

fn from_screen_coords(pos: Vec2) -> (usize, usize) {
    (pos.x as usize, pos.y as usize)
}

pub(crate) fn handle_keyboard_input(camera: &mut Camera) {
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
}

pub(crate) fn handle_pointer_input(
    commands: &mut Vec<Command>,
    submarine: &SubmarineState,
    sub_index: usize,
    mutable_resources: &mut MutableSubResources,
    game_settings: &mut GameSettings,
) {
    let GameSettings {
        camera,
        current_tool,
        dragging_object,
        highlighting_settings,
        ..
    } = game_settings;

    let draw_egui = &mut game_settings.draw_settings.draw_egui;

    let macroquad_camera = camera.to_macroquad_camera(Some(submarine.navigation.position));
    let mouse_position = mouse_position();

    camera.pointing_at =
        from_screen_coords(macroquad_camera.screen_to_world(mouse_position.into()));

    let settings_button = Rect::new(10.0, 10.0, 20.0, 20.0);
    if !*draw_egui {
        *highlighting_settings = settings_button.contains(mouse_position.into());
    }

    if is_mouse_button_pressed(MouseButton::Left) && !*draw_egui && *highlighting_settings {
        *draw_egui = true;
    }

    let scrolling = if is_mouse_button_down(MouseButton::Right) {
        true
    } else {
        matches!(current_tool, Tool::Interact) && is_mouse_button_down(MouseButton::Left)
    };

    if is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Right) {
        camera.dragging_from = mouse_position;
    }

    if scrolling && !*dragging_object {
        let new_position = mouse_position;

        let old = macroquad_camera.screen_to_world(Vec2::from(camera.dragging_from));
        let new = macroquad_camera.screen_to_world(Vec2::from(new_position));

        let delta = new - old;

        camera.offset_x += delta.x;
        camera.offset_y += delta.y;

        camera.dragging_from = mouse_position;
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

    let mouse_position = macroquad_camera.screen_to_world(mouse_position.into());

    mutable_resources.highlighting_object = None;

    if is_mouse_button_down(MouseButton::Left) && !*dragging_object {
        let (width, height) = submarine.water_grid.size();
        let (x, y) = camera.pointing_at;

        if x >= width || y >= height {
            return;
        }

        let cell_command = match *current_tool {
            Tool::Interact => None,
            Tool::EditWater { add } => Some(CellCommand::EditWater { add }),
            Tool::EditWalls { add } => Some(CellCommand::EditWalls { add }),
            Tool::EditWires { color } => Some(CellCommand::EditWires { color }),
        };

        if let Some(cell_command) = cell_command {
            commands.push(Command::Cell {
                cell_command,
                cell: (x, y),
                submarine_id: sub_index,
            });
        }
    }

    if let Tool::Interact = current_tool {
        interact(
            commands,
            submarine,
            sub_index,
            mouse_position,
            game_settings,
            mutable_resources,
        );
    }
}

fn interact(
    commands: &mut Vec<Command>,
    submarine: &SubmarineState,
    sub_index: usize,
    mouse_position: Vec2,
    game_settings: &mut GameSettings,
    mutable_resources: &mut MutableSubResources,
) {
    let camera = &mut game_settings.camera;
    let dragging_object = &mut game_settings.dragging_object;

    for (obj_index, object) in submarine.objects.iter().enumerate() {
        let draw_rect = object_rect(object);

        if draw_rect.contains(mouse_position) {
            mutable_resources.highlighting_object = Some(obj_index);

            let hover_position = mouse_position - draw_rect.point();

            hover_over_object(object, (hover_position.x, hover_position.y));

            if is_mouse_button_pressed(MouseButton::Left) {
                commands.push(Command::Interact {
                    submarine_id: sub_index,
                    object_id: obj_index,
                });
                *dragging_object = true;
                return;
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        *dragging_object = false;
    }

    // Placing an object
    let (width, height) = submarine.water_grid.size();
    if let Some(placing_object) = &mut game_settings.placing_object {
        let (x, y) = camera.pointing_at;

        let size = object_size(&placing_object.object_type);

        let x = x.wrapping_sub(size.0 / 2 + size.0 % 2);
        let y = y.wrapping_sub(size.1 / 2 + size.1 % 2 + 1);

        if x < width && y < height {
            placing_object.submarine = sub_index;
            placing_object.position = Some((x, y));

            if is_mouse_button_pressed(MouseButton::Left) {
                commands.push(Command::Cell {
                    cell_command: CellCommand::AddObject {
                        object_type: placing_object.object_type.clone(),
                    },
                    cell: (x, y),
                    submarine_id: sub_index,
                });

                if !is_key_down(KeyCode::LeftShift) && !is_key_down(KeyCode::RightShift) {
                    game_settings.placing_object = None;
                }

                // Prevent placing water right after clicking
                *dragging_object = true;
            }

            if is_mouse_button_down(MouseButton::Right) {
                game_settings.placing_object = None;
            }
        }

        return;
    }
}
