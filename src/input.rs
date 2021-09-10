use macroquad::prelude::{
    is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed,
    is_mouse_button_released, mouse_position, mouse_wheel, KeyCode, MouseButton, Rect, Vec2,
};

use crate::{
    app::{GameSettings, Tool},
    draw::{object_rect, object_size, Camera},
    game_state::{
        objects::{Object, ObjectType},
        state::{Navigation, SubmarineState},
    },
    game_state::{
        update::{CellCommand, Command},
        wires::WireColor,
    },
    resources::MutableSubResources,
};

pub(crate) enum Dragging {
    Camera,
    Nothing,
    Wires {
        color: WireColor,
        dragging_from: (usize, usize),
    },
    Tool(Tool),
}

fn from_screen_coords(pos: Vec2) -> (usize, usize) {
    (pos.x as usize, pos.y as usize)
}

// Only called when egui doesn't want the keyboard
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
    if is_key_pressed(KeyCode::Escape) {
        *current_tool = Tool::Interact;
    }
}

// Only called when egui doesn't want the mouse/touch pointer
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
        dragging,
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
        return;
    }

    let right_click_dragging = is_mouse_button_down(MouseButton::Right);
    let dragging_camera = matches!(dragging, Some(Dragging::Camera)) || right_click_dragging;

    if dragging_camera {
        let new_position = mouse_position;

        let old = macroquad_camera.screen_to_world(Vec2::from(camera.dragging_from));
        let new = macroquad_camera.screen_to_world(Vec2::from(new_position));

        let delta = new - old;

        camera.offset_x += delta.x;
        camera.offset_y += delta.y;
    }

    camera.dragging_from = mouse_position;

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
    mutable_resources.sub_cursor = mouse_position.into();

    // Highlight current object.
    // Also, some objects react by just hovering over them.
    let clicked = false;
    interact(
        commands,
        submarine,
        sub_index,
        mouse_position,
        mutable_resources,
        clicked,
    );

    // Ghost of object being placed, if any
    if let Tool::PlaceObject(placing_object) = current_tool {
        let (x, y) = camera.pointing_at;

        let size = object_size(&placing_object.object_type);

        let (width, height) = submarine.water_grid.size();
        let x = x.wrapping_sub(size.0 / 2 + size.0 % 2);
        let y = y.wrapping_sub(size.1 / 2 + size.1 % 2 + 1);

        if x < width && y < height {
            placing_object.submarine = sub_index;
            placing_object.position = Some((x, y));
        }
    }

    // Press
    if is_mouse_button_pressed(MouseButton::Left) {
        *dragging = Some(match current_tool {
            Tool::Interact => {
                let clicked = true;
                let clicked_object = interact(
                    commands,
                    submarine,
                    sub_index,
                    mouse_position,
                    mutable_resources,
                    clicked,
                );

                if clicked_object {
                    Dragging::Nothing
                } else {
                    Dragging::Camera
                }
            }
            Tool::PlaceObject(placing_object) => {
                if let Some(position) = placing_object.position {
                    commands.push(Command::Cell {
                        cell_command: CellCommand::AddObject {
                            object_type: placing_object.object_type.clone(),
                        },
                        cell: position,
                        submarine_id: placing_object.submarine,
                    });
                }

                let place_more_objects =
                    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                if !place_more_objects {
                    *current_tool = Tool::Interact;
                }

                Dragging::Nothing
            }
            Tool::EditWires { color } => Dragging::Wires {
                color: *color,
                dragging_from: camera.pointing_at,
            },
            tool => Dragging::Tool(tool.clone()),
        });
    }

    // Hold
    if let Some(Dragging::Tool(tool)) = dragging {
        let (x, y) = camera.pointing_at;
        let (width, height) = submarine.water_grid.size();

        if x < width || y < height {
            let cell_command = match *tool {
                Tool::Interact => None,
                Tool::EditWater { add } => Some(CellCommand::EditWater { add }),
                Tool::EditWalls { add } => Some(CellCommand::EditWalls { add }),
                Tool::EditWires { .. } => None,
                Tool::PlaceObject(_) => None,
            };

            if let Some(cell_command) = cell_command {
                commands.push(Command::Cell {
                    cell_command,
                    cell: (x, y),
                    submarine_id: sub_index,
                });
            }
        }
    }

    // Release
    if is_mouse_button_released(MouseButton::Left) {
        if let Some(Dragging::Wires {
            color,
            dragging_from,
        }) = dragging.take()
        {
            let (width, height) = submarine.water_grid.size();
            let (mut start_x, mut start_y) = dragging_from;
            let (mut end_x, mut end_y) = camera.pointing_at;

            if start_x == end_x || start_y == end_y {
                if start_x > end_x {
                    std::mem::swap(&mut start_x, &mut end_x);
                }

                if start_y > end_y {
                    std::mem::swap(&mut start_y, &mut end_y)
                }

                let mut add = false;

                'check: for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        if (x < width || y < height)
                            && !submarine.wire_grid.cell(x, y).value(color).connected()
                        {
                            add = true;
                            break 'check;
                        }
                    }
                }

                for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        if x < width || y < height {
                            let cell_command = CellCommand::EditWires { color, add };

                            commands.push(Command::Cell {
                                cell_command,
                                cell: (x, y),
                                submarine_id: sub_index,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn hovering_over_sonar(
    object: &Object,
    obj_index: usize,
    hover_position: Vec2,
    mutable_resources: &mut MutableSubResources,
) -> bool {
    if let ObjectType::Sonar { active: true, .. } = &object.object_type {
        if !object.powered {
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
    mutable_resources: &mut MutableSubResources,
    clicked: bool,
) -> bool {
    mutable_resources.sonar_cursor = None;

    mutable_resources.highlighting_object = None;

    for (obj_index, object) in submarine.objects.iter().enumerate() {
        let draw_rect = object_rect(object);

        if !draw_rect.contains(mouse_position) {
            continue;
        }

        mutable_resources.highlighting_object = Some(obj_index);

        let hover_position = mouse_position - draw_rect.point();

        let hovering_over_sonar =
            hovering_over_sonar(object, obj_index, hover_position, mutable_resources);

        if !clicked {
            return false;
        }

        let command = if hovering_over_sonar {
            Command::SetSonarTarget {
                submarine_id: sub_index,
                object_id: obj_index,
                rock_position: sonar_target(&submarine.navigation, mutable_resources),
            }
        } else {
            Command::Interact {
                submarine_id: sub_index,
                object_id: obj_index,
            }
        };

        commands.push(command);
        return true;
    }

    false
}
