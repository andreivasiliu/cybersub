use serde::{Deserialize, Serialize};

use crate::game_state::{
    collisions::{update_rock_collisions, update_submarine_collisions},
    objects::{interact_with_object, update_objects, Object, ObjectType},
    sonar::{update_sonar, Sonar},
    state::{GameState, Navigation, SubmarineState, SubmarineTemplate, UpdateSettings},
    water::WaterGrid,
    wires::{WireColor, WireGrid},
};

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
                });

                events.push(UpdateEvent::SubmarineCreated);
            }
        }
    }

    let update_settings = &game_state.update_settings;

    for submarine in &mut game_state.submarines {
        submarine.collisions.clear();
    }

    for (sub_index, submarine) in game_state.submarines.iter_mut().enumerate() {
        if update_settings.update_position {
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

            navigation.speed.0 =
                (navigation.speed.0 + navigation.acceleration.0).clamp(-2048, 2048);
            navigation.speed.1 =
                (navigation.speed.1 + navigation.acceleration.1).clamp(-2048, 2048);

            navigation.position.0 += navigation.speed.0 / 256;
            navigation.position.1 += navigation.speed.1 / 256;
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
