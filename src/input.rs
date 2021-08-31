use macroquad::prelude::{
    is_key_down, is_mouse_button_down, is_mouse_button_pressed, is_mouse_button_released,
    mouse_position, mouse_wheel, KeyCode, MouseButton, Rect, Vec2,
};

use crate::{
    app::{GameSettings, Tool},
    draw::{object_rect, object_size, Camera},
    game_state::update::{CellCommand, Command},
    game_state::{
        objects::{Object, ObjectType},
        state::{Navigation, SubmarineState},
    },
    resources::MutableSubResources,
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

    let (width, height) = submarine.water_grid.size();
    let sub_rect = Rect::new(0.0, 0.0, width as f32, height as f32);

    let mouse_position = macroquad_camera.screen_to_world(mouse_position.into());
    mutable_resources.sub_cursor = if sub_rect.contains(mouse_position) {
        Some(mouse_position.into())
    } else {
        None
    };

    mutable_resources.highlighting_object = None;

    if is_mouse_button_down(MouseButton::Left) && !*dragging_object {
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

fn hovering_over_sonar(
    object: &Object,
    obj_index: usize,
    hover_position: Vec2,
    mutable_resources: &mut MutableSubResources,
) -> bool {
    if let ObjectType::Sonar {
        active: true,
        powered,
        ..
    } = &object.object_type
    {
        if !powered {
            // Acknowledge hovering, but don't set target
            return true;
        }
        let sonar_middle = (9.5, 7.5);
        let cursor = (
            hover_position.x - sonar_middle.0,
            hover_position.y - sonar_middle.1,
        );

        let length_squared = cursor.0 * cursor.0 + cursor.1 * cursor.1;

        if length_squared < 5.0 * 5.0 {
            mutable_resources.sonar_cursor = Some((obj_index, cursor));
            return true;
        }
    }

    false
}

fn sonar_target(
    navigation: &Navigation,
    mutable_resources: &MutableSubResources,
) -> (usize, usize) {
    if let Some((_obj, target)) = mutable_resources.sonar_cursor {
        // 16 sub-cells per rock-cell, 16 movement points per rock-cell
        let world_ratio = 16.0 * 16.0;
        // 75 rock-cells radius, on 6-pixels per cell resolution
        let sonar_ratio = 75.0 / 6.0;

        let target_x = navigation.position.0 + (target.0 * world_ratio * sonar_ratio) as i32;
        let target_y = navigation.position.1 + (target.1 * world_ratio * sonar_ratio) as i32;

        (target_x as usize, target_y as usize)
    } else {
        unreachable!("Checked by hovering_over_sonar()")
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

    mutable_resources.sonar_cursor = None;

    for (obj_index, object) in submarine.objects.iter().enumerate() {
        let draw_rect = object_rect(object);

        if draw_rect.contains(mouse_position) {
            mutable_resources.highlighting_object = Some(obj_index);

            let hover_position = mouse_position - draw_rect.point();

            let clicked = is_mouse_button_pressed(MouseButton::Left);
            let hovering_over_sonar =
                hovering_over_sonar(object, obj_index, hover_position, mutable_resources);

            if hovering_over_sonar && clicked {
                commands.push(Command::SetSonarTarget {
                    submarine_id: sub_index,
                    object_id: obj_index,
                    rock_position: sonar_target(&submarine.navigation, mutable_resources),
                });
                return;
            } else if clicked {
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
