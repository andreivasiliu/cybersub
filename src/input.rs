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
        dragging_from_tile: (usize, usize),
        dragging_from_sub: usize,
    },
    Tool(Tool),
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
    game_settings: &mut GameSettings,
    submarines: &[crate::game_state::state::SubmarineState],
    mutable_sub_resources: &mut [MutableSubResources],
) {
    let GameSettings {
        camera,
        highlighting_settings,
        dragging,
        ..
    } = game_settings;

    let mouse_position = mouse_position();
    let world_camera = camera.to_macroquad_camera(None);
    camera.pointing_at_world = world_camera.screen_to_world(mouse_position.into()).into();

    // If egui is disabled, then the only UI is an icon to turn it back on.
    let draw_egui = &mut game_settings.draw_settings.draw_egui;

    let settings_button = Rect::new(10.0, 10.0, 20.0, 20.0);
    if !*draw_egui {
        *highlighting_settings = settings_button.contains(mouse_position.into());
    }

    if is_mouse_button_pressed(MouseButton::Left) && !*draw_egui && *highlighting_settings {
        *draw_egui = true;
        return;
    }

    // Mouse panning
    let right_click_dragging = is_mouse_button_down(MouseButton::Right);
    let dragging_camera = matches!(dragging, Some(Dragging::Camera)) || right_click_dragging;

    if dragging_camera {
        let new_position = mouse_position;

        let old = world_camera.screen_to_world(Vec2::from(camera.dragging_from));
        let new = world_camera.screen_to_world(Vec2::from(new_position));

        let delta = new - old;

        camera.offset_x += delta.x;
        camera.offset_y += delta.y;
    }

    camera.dragging_from = mouse_position;

    // Mouse zooming
    let scroll = mouse_wheel().1;
    if scroll != 0.0 {
        let multiplier = if cfg!(target_arch = "wasm32") {
            0.1
        } else {
            1.0
        };

        camera.zoom = (camera.zoom + (scroll * multiplier) as i32 * 4).clamp(-512, 36);
    }

    // Ghost of submarine being placed, if any
    if let Tool::PlaceSubmarine {
        template_id,
        position,
        ..
    } = &mut game_settings.current_tool
    {
        if let Some((_name, template)) = game_settings.submarine_templates.get(*template_id) {
            let pointer_offset = (
                camera.pointing_at_world.0 * 16.0,
                camera.pointing_at_world.1 * 16.0,
            );

            let new_sub_middle = (
                template.size.0 as i32 * 16 / 2,
                template.size.1 as i32 * 16 / 2,
            );

            let new_sub_position = (
                (pointer_offset.0 as i32 - new_sub_middle.0).max(0) as usize,
                (pointer_offset.1 as i32 - new_sub_middle.1).max(0) as usize,
            );

            *position = Some(new_sub_position);
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some((_name, template)) = game_settings.submarine_templates.get(*template_id) {
                if let Some(position) = position {
                    commands.push(Command::CreateSubmarine {
                        submarine_template: Box::new(template.clone()),
                        rock_position: *position,
                    });
                }
            }

            game_settings.current_tool = Tool::Interact;
            game_settings.dragging = None;

            return;
        }
    }

    // Record mouse position relative to each submarine.
    let submarines_and_resources = submarines.iter().zip(mutable_sub_resources.iter_mut());
    for (submarine, mutable_resources) in submarines_and_resources {
        let macroquad_camera = camera.to_macroquad_camera(Some(submarine.navigation.position));

        let mouse_position = macroquad_camera.screen_to_world(mouse_position.into());
        mutable_resources.sub_cursor = mouse_position.into();

        let grid_coords = (mouse_position.x as usize, mouse_position.y as usize);

        let (width, height) = submarine.water_grid.size();

        let inside_grid = mutable_resources.sub_cursor.0 >= 0.0
            && mutable_resources.sub_cursor.1 >= 0.0
            && grid_coords.0 < width
            && grid_coords.1 < height;

        mutable_resources.highlighting_object = None;

        mutable_resources.sub_cursor_tile = if inside_grid { Some(grid_coords) } else { None };
    }

    // Do input actions only on one submarine, preferably one with a grid
    // under the mouse.
    let submarines_and_resources = submarines.iter().zip(mutable_sub_resources).enumerate().rev();
    for (sub_index, (submarine, mutable_resources)) in submarines_and_resources {
        if let Some(sub_cursor_tile) = mutable_resources.sub_cursor_tile {
            if handle_pointer_input_on_submarine(
                commands,
                submarine,
                sub_index,
                mutable_resources,
                game_settings,
                sub_cursor_tile,
            ) {
                break;
            }
        }
    }
}

// Only called when the cursor is on a tile of this submarine
pub(crate) fn handle_pointer_input_on_submarine(
    commands: &mut Vec<Command>,
    submarine: &SubmarineState,
    sub_index: usize,
    mutable_resources: &mut MutableSubResources,
    game_settings: &mut GameSettings,
    sub_cursor_tile: (usize, usize),
) -> bool {
    let mut actioned = false;

    let GameSettings {
        current_tool,
        dragging,
        ..
    } = game_settings;

    // Highlight current object.
    // Also, some objects react by just hovering over them.
    let clicked = false;
    interact(commands, submarine, sub_index, mutable_resources, clicked);

    // Ghost of object being placed, if any
    if let Tool::PlaceObject(placing_object) = current_tool {
        let (x, y) = sub_cursor_tile;

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
        actioned = true;

        *dragging = Some(match current_tool {
            Tool::Interact => {
                let clicked = true;
                let clicked_object =
                    interact(commands, submarine, sub_index, mutable_resources, clicked);

                if clicked_object {
                    Dragging::Nothing
                } else {
                    // Disacknowledge the click if no object was interacted
                    // with, to let other subs see the click.
                    actioned = false;

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
            Tool::PlaceSubmarine { .. } => Dragging::Nothing,
            Tool::EditWires { color } => Dragging::Wires {
                color: *color,
                dragging_from_tile: sub_cursor_tile,
                dragging_from_sub: sub_index,
            },
            tool @ Tool::EditWater { .. } => Dragging::Tool(tool.clone()),
            tool @ Tool::EditWalls { .. } => Dragging::Tool(tool.clone()),
        });
    }

    // Hold
    if let Some(Dragging::Tool(tool)) = dragging {
        let cell_command = match *tool {
            Tool::Interact => None,
            Tool::EditWater { add } => Some(CellCommand::EditWater { add }),
            Tool::EditWalls { add } => Some(CellCommand::EditWalls { add }),
            Tool::EditWires { .. } => None,
            Tool::PlaceObject(_) => None,
            Tool::PlaceSubmarine { .. } => None,
        };

        if let Some(cell_command) = cell_command {
            commands.push(Command::Cell {
                cell_command,
                cell: sub_cursor_tile,
                submarine_id: sub_index,
            });
        }
    }

    // Release
    if is_mouse_button_released(MouseButton::Left) {
        if let Some(Dragging::Wires {
            color,
            dragging_from_tile,
            dragging_from_sub,
        }) = dragging.take()
        {
            actioned = true;

            if dragging_from_sub == sub_index {
                let (width, height) = submarine.water_grid.size();
                let (start_x, start_y) = dragging_from_tile;
                let (end_x, end_y) = sub_cursor_tile;

                let x_length = (start_x as i32 - end_x as i32).abs();
                let y_length = (start_y as i32 - end_y as i32).abs();

                let (mut start_x, mut start_y, mut end_x, mut end_y) = if x_length > y_length {
                    (start_x, start_y, end_x, start_y)
                } else {
                    (start_x, start_y, start_x, end_y)
                };

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

    actioned
}

fn hovering_over_sonar(object: &Object, hover_position: Vec2) -> Option<(f32, f32)> {
    if let ObjectType::Sonar { active: true, .. } = &object.object_type {
        let sonar_middle = (9.5, 7.5);
        let cursor = (
            hover_position.x - sonar_middle.0,
            hover_position.y - sonar_middle.1,
        );

        let length_squared = cursor.0 * cursor.0 + cursor.1 * cursor.1;

        if length_squared < 5.0 * 5.0 {
            return Some(cursor);
        }
    }

    None
}

fn sonar_target(navigation: &Navigation, sonar_cursor: (f32, f32)) -> (usize, usize) {
    // 16 sub-cells per rock-cell, 16 movement points per rock-cell
    let world_ratio = 16.0 * 16.0;
    // 75 rock-cells radius, on 6-pixels per cell resolution
    let sonar_ratio = 75.0 / 6.0;

    let target_x = navigation.position.0 + (sonar_cursor.0 * world_ratio * sonar_ratio) as i32;
    let target_y = navigation.position.1 + (sonar_cursor.1 * world_ratio * sonar_ratio) as i32;

    (target_x as usize, target_y as usize)
}

fn interact(
    commands: &mut Vec<Command>,
    submarine: &SubmarineState,
    sub_index: usize,
    mutable_resources: &mut MutableSubResources,
    clicked: bool,
) -> bool {
    mutable_resources.sonar_cursor = None;

    mutable_resources.highlighting_object = None;

    let mouse_position: Vec2 = mutable_resources.sub_cursor.into();

    for (obj_index, object) in submarine.objects.iter().enumerate() {
        let draw_rect = object_rect(object);

        if !draw_rect.contains(mouse_position) {
            continue;
        }

        mutable_resources.highlighting_object = Some(obj_index);

        let hover_position = mouse_position - draw_rect.point();

        if let Some(cursor) = hovering_over_sonar(object, hover_position) {
            mutable_resources.sonar_cursor = Some((obj_index, cursor));

            if clicked && object.powered {
                commands.push(Command::SetSonarTarget {
                    submarine_id: sub_index,
                    object_id: obj_index,
                    rock_position: sonar_target(&submarine.navigation, cursor),
                });
                return true;
            }
        }

        if clicked {
            commands.push(Command::Interact {
                submarine_id: sub_index,
                object_id: obj_index,
            });
        }

        // Don't acknowledge the click if it's a docking connector; this allows
        // interacting with multiple connectors on multiple subs that overlap
        // each other.
        if let ObjectType::DockingConnectorTop { .. } = object.object_type {
            return false;
        }
        if let ObjectType::DockingConnectorBottom { .. } = object.object_type {
            return false;
        }

        return true;
    }

    false
}
