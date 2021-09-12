use serde::{Deserialize, Serialize};

use crate::game_state::{
    collisions::{update_rock_collisions, update_submarine_collisions},
    objects::{interact_with_object, update_objects, Object, ObjectType},
    sonar::{update_sonar, Sonar},
    state::{GameState, Navigation, SubmarineState, SubmarineTemplate, UpdateSettings},
    water::WaterGrid,
    wires::{WireColor, WireGrid},
};

use super::state::{DockingDirection, DockingPoint};

/// A request to mutate state. Created by the UI and player actions.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum Command {
    Interact {
        submarine_id: usize,
        object_id: usize,
    },
    Cell {
        submarine_id: usize,
        cell: (usize, usize),
        cell_command: CellCommand,
    },
    ClearWater {
        submarine_id: usize,
    },
    ChangeUpdateSettings {
        update_settings: UpdateSettings,
    },
    SetSonarTarget {
        submarine_id: usize,
        object_id: usize,
        rock_position: (usize, usize),
    },
    CreateSubmarine {
        submarine_template: Box<SubmarineTemplate>,
        rock_position: (usize, usize),
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum CellCommand {
    EditWires { add: bool, color: WireColor },
    EditWalls { add: bool },
    EditWater { add: bool },
    AddObject { object_type: ObjectType },
}

pub(crate) enum UpdateEvent {
    Submarine {
        submarine_id: usize,
        submarine_event: SubmarineUpdatedEvent,
    },
    SubmarineCreated,
    GameStateReset,
}

pub(crate) enum SubmarineUpdatedEvent {
    Sonar,
    Walls,
    Wires,
    Signals,
}

pub(crate) fn update_game(
    commands: impl Iterator<Item = Command>,
    game_state: &mut GameState,
    events: &mut Vec<UpdateEvent>,
) {
    game_state.collisions.clear();

    update_state_from_commands(commands, game_state, events);

    let update_settings = &game_state.update_settings;

    for submarine in &mut game_state.submarines {
        submarine.collisions.clear();
    }

    update_docking_points(&mut game_state.submarines);

    for (sub_index, submarine) in game_state.submarines.iter_mut().enumerate() {
        if update_settings.update_position {
            update_navigation(submarine);
        }

        if update_settings.update_water {
            submarine.water_grid.update(
                update_settings.enable_gravity,
                update_settings.enable_inertia,
            );
        }
        if update_settings.update_wires {
            for _ in 0..3 {
                let mut signals_updated = false;
                submarine.wire_grid.update(&mut signals_updated);

                if signals_updated {
                    events.push(UpdateEvent::Submarine {
                        submarine_id: sub_index,
                        submarine_event: SubmarineUpdatedEvent::Signals,
                    });
                }
            }

            submarine.wire_grid.update_bundles();
        }
        if update_settings.update_objects {
            let mut walls_updated = false;
            update_objects(submarine, &mut walls_updated);

            if walls_updated {
                events.push(UpdateEvent::Submarine {
                    submarine_id: sub_index,
                    submarine_event: SubmarineUpdatedEvent::Walls,
                });
            }
        }
        if update_settings.update_sonar {
            let updated = update_sonar(
                &mut submarine.sonar,
                &submarine.navigation,
                submarine.water_grid.size(),
                &game_state.rock_grid,
            );

            if updated {
                events.push(UpdateEvent::Submarine {
                    submarine_id: sub_index,
                    submarine_event: SubmarineUpdatedEvent::Sonar,
                });
            }
        }

        if update_settings.update_collision {
            game_state.collisions.clear();
            update_rock_collisions(submarine, &game_state.rock_grid, &mut game_state.collisions);
        }
    }

    if update_settings.update_position {
        update_position(&mut game_state.submarines);
    }

    if update_settings.update_collision {
        for sub1_index in 0..game_state.submarines.len() {
            for sub2_index in sub1_index + 1..game_state.submarines.len() {
                let (left, right) = game_state.submarines.split_at_mut(sub2_index);
                let submarine1 = &mut left[sub1_index];
                let submarine2 = &mut right[0];

                update_submarine_collisions(submarine1, submarine2);
                update_submarine_collisions(submarine2, submarine1);
            }
        }
    }
}

fn update_state_from_commands(
    commands: impl Iterator<Item = Command>,
    game_state: &mut GameState,
    events: &mut Vec<UpdateEvent>,
) {
    for command in commands {
        match command {
            Command::Interact {
                submarine_id,
                object_id,
            } => {
                if let Some(submarine) = game_state.submarines.get_mut(submarine_id) {
                    if let Some(object) = submarine.objects.get_mut(object_id) {
                        interact_with_object(object);
                    }
                };
            }
            Command::Cell {
                submarine_id,
                cell,
                cell_command,
            } => {
                if let Some(submarine) = game_state.submarines.get_mut(submarine_id) {
                    let (width, height) = submarine.water_grid.size();
                    if cell.0 >= width || cell.1 >= height {
                        continue;
                    }

                    let water_cell = submarine.water_grid.cell_mut(cell.0, cell.1);

                    match &cell_command {
                        CellCommand::EditWater { add: true } => water_cell.fill(),
                        CellCommand::EditWater { add: false } => water_cell.empty(),
                        CellCommand::EditWalls { add: true } => water_cell.make_wall(),
                        CellCommand::EditWalls { add: false } => water_cell.clear_wall(),
                        CellCommand::EditWires { add: true, color } => {
                            submarine.wire_grid.make_wire(cell.0, cell.1, *color)
                        }
                        CellCommand::EditWires { add: false, color } => {
                            submarine.wire_grid.clear_wire(cell.0, cell.1, *color)
                        }
                        CellCommand::AddObject { object_type } => {
                            submarine.objects.push(Object {
                                object_type: object_type.clone(),
                                position: (cell.0 as u32, cell.1 as u32),
                                powered: false,
                            });
                        }
                    }

                    match &cell_command {
                        CellCommand::EditWater { .. } | CellCommand::EditWalls { .. } => {
                            events.push(UpdateEvent::Submarine {
                                submarine_id,
                                submarine_event: SubmarineUpdatedEvent::Walls,
                            });
                        }
                        CellCommand::EditWires { .. } => {
                            events.push(UpdateEvent::Submarine {
                                submarine_id,
                                submarine_event: SubmarineUpdatedEvent::Wires,
                            });
                        }
                        CellCommand::AddObject { .. } => (),
                    }
                }
            }
            Command::ClearWater { submarine_id } => {
                if let Some(submarine) = game_state.submarines.get_mut(submarine_id) {
                    submarine.water_grid.clear();
                }
            }
            Command::ChangeUpdateSettings { update_settings } => {
                game_state.update_settings = update_settings
            }
            Command::SetSonarTarget {
                submarine_id,
                object_id,
                rock_position,
            } => {
                if let Some(submarine) = game_state.submarines.get_mut(submarine_id) {
                    if let Some(object) = submarine.objects.get_mut(object_id) {
                        if let ObjectType::Sonar {
                            navigation_target, ..
                        } = &mut object.object_type
                        {
                            *navigation_target = Some(rock_position);
                        }
                    }
                };
            }
            Command::CreateSubmarine {
                submarine_template,
                rock_position,
            } => {
                let (width, height) = submarine_template.size;
                let position = (rock_position.0 as i32, rock_position.1 as i32);
                game_state.submarines.push(SubmarineState {
                    background_pixels: submarine_template.background_pixels,
                    water_grid: WaterGrid::from_cells(
                        width,
                        height,
                        &submarine_template.water_cells,
                    ),
                    wire_grid: WireGrid::from_wire_points(
                        width,
                        height,
                        &submarine_template.wire_points,
                    ),
                    objects: submarine_template.objects,
                    navigation: Navigation {
                        position,
                        target: position,
                        ..Default::default()
                    },
                    sonar: Sonar::default(),
                    collisions: Vec::new(),
                    docking_points: Vec::new(),
                });

                events.push(UpdateEvent::SubmarineCreated);
            }
        }
    }
}

fn update_docking_points(submarines: &mut [SubmarineState]) {
    for submarine in submarines.iter_mut() {
        submarine.docking_points.clear();

        for (obj_index, object) in submarine.objects.iter_mut().enumerate() {
            // if !object.powered {
            //     continue;
            // }

            let (connected, direction) = match &mut object.object_type {
                ObjectType::DockingConnectorTop { connected, .. } => (connected, DockingDirection::Top),
                ObjectType::DockingConnectorBottom { connected, .. } => (connected, DockingDirection::Bottom),
                _ => continue,
            };

            *connected = false;

            let vertical_offset = match direction {
                DockingDirection::Top => 3 * 16,
                DockingDirection::Bottom => 7 * 16,
            };

            let connection_point = (
                submarine.navigation.position.0 + object.position.0 as i32 * 16 + 11 * 16,
                submarine.navigation.position.1 + object.position.1 as i32 * 16 + vertical_offset,
            );

            submarine.docking_points.push(DockingPoint {
                connection_point,
                connector_object_id: obj_index,
                connected_to: None,
                in_proximity_to: None,
                speed_offset: (0, 0),
                direction,
            });
        }
    }

    // Attempt to nudge subs closer to each other if docking points are powered
    // and in proximity.
    for sub1_index in 0..submarines.len() {
        let (left_subs, right_subs) = submarines.split_at_mut(sub1_index + 1);

        for sub2_index_offset in 0..right_subs.len() {
            let sub1 = &mut left_subs[sub1_index];
            let sub2 = &mut right_subs[sub2_index_offset];

            for point1 in &mut sub1.docking_points {
                for point2 in &mut sub2.docking_points {
                    if point1.in_proximity_to.is_some() || point2.in_proximity_to.is_some() {
                        continue;
                    }

                    match (point1.direction, point2.direction) {
                        (DockingDirection::Top, DockingDirection::Bottom) => (),
                        (DockingDirection::Bottom, DockingDirection::Top) => (),
                        _ => continue,
                    };

                    let diff_x = point1.connection_point.0 - point2.connection_point.0;
                    let diff_y = point1.connection_point.1 - point2.connection_point.1;

                    if diff_x.abs() >= 128 || diff_y.abs() >= 128 {
                        continue;
                    }

                    point1.in_proximity_to = Some(point2.connection_point);
                    point2.in_proximity_to = Some(point1.connection_point);

                    let speed_x = diff_x.clamp(-2, 2);
                    let speed_y = diff_y.clamp(-2, 2);

                    // Maximize chances of reaching the exact connecting point
                    point1.speed_offset = (-speed_x / 2 + speed_x % 2, -speed_y / 2 + speed_y % 2);
                    point2.speed_offset = (speed_x / 2, speed_y / 2);

                    dbg!(diff_x, diff_y);

                    if diff_x.abs() >= 4 || diff_y.abs() >= 4 {
                        continue;
                    }

                    let sub2_index = sub1_index + 1 + sub2_index_offset;
                    point1.connected_to = Some((sub2_index, point2.connector_object_id));
                    point2.connected_to = Some((sub1_index, point1.connector_object_id));

                    match &mut sub1.objects[point1.connector_object_id].object_type
                    {
                        ObjectType::DockingConnectorTop { connected, .. } => *connected = true,
                        ObjectType::DockingConnectorBottom { connected, .. } => *connected = true,
                        _ => unreachable!("Object type checked above"),
                    };

                    match &mut sub2.objects[point2.connector_object_id].object_type
                    {
                        ObjectType::DockingConnectorTop { connected, .. } => *connected = true,
                        ObjectType::DockingConnectorBottom { connected, .. } => *connected = true,
                        _ => unreachable!("Object type checked above"),
                    };
                }
            }
        }
    }
}

fn update_navigation(submarine: &mut SubmarineState) {
    let navigation = &mut submarine.navigation;

    // Compute weight based on number of walls
    let weight = submarine.water_grid.total_walls() as i32;

    // Compute buoyancy; the numbers are just random stuff that seems to
    // somewhat work for both the Dugong and the Bunyip
    let mut buoyancy = 0;
    buoyancy -= weight * 16;
    buoyancy += submarine.water_grid.total_inside() as i32 * 13;
    buoyancy -= submarine.water_grid.total_water() as i32 * 16 / 1024;

    // Massive submarines are harder to move
    let mass = (weight * weight / 1500 / 1500).max(1);

    let y_acceleration = (buoyancy * weight) / 1024 / 100;
    navigation.acceleration.1 = -y_acceleration / 8 / mass;

    navigation.speed.0 = (navigation.speed.0 + navigation.acceleration.0).clamp(-2048, 2048);
    navigation.speed.1 = (navigation.speed.1 + navigation.acceleration.1).clamp(-2048, 2048);

    // Speed overrides from docking connectors that are trying to dock
    navigation.docking_override = (0, 0);

    for point in &submarine.docking_points {
        navigation.docking_override.0 += point.speed_offset.0;
        navigation.docking_override.1 += point.speed_offset.1;
    }
}

fn update_position(submarines: &mut [SubmarineState]) {
    let mut submarine_group = Vec::new();
    let mut group_speed = Vec::new();
    let mut group_members = Vec::new();

    submarine_group.resize(submarines.len(), None);

    // Assign submarines to groups if they're docked together
    for (sub_index, submarine) in submarines.iter().enumerate() {
        let group = *submarine_group[sub_index]
            .get_or_insert_with(|| {
                group_speed.push((0, 0));
                group_members.push(0);
                group_speed.len() - 1
            });

        for point in &submarine.docking_points {
            if let Some((connected_sub_index, _obj_id)) = point.connected_to {
                submarine_group[connected_sub_index] = Some(group);
            }
        }

        group_members[group] += 1;
        group_speed[group].0 += submarine.navigation.speed.0;
        group_speed[group].1 += submarine.navigation.speed.1;
    }

    // Apply the same speed to the whole docked group
    // FIXME: should use weighted average so tiny subs can't pull big subs so easily
    for (sub_index, submarine) in submarines.iter_mut().enumerate() {
        let group = submarine_group[sub_index].expect("Grouped above");
        submarine.navigation.position.0 += group_speed[group].0 / group_members[group] / 256;
        submarine.navigation.position.1 += group_speed[group].1 / group_members[group] / 256;
        submarine.navigation.position.0 += submarine.navigation.docking_override.0;
        submarine.navigation.position.1 += submarine.navigation.docking_override.1;
    }
}