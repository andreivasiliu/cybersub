use crate::{
    app::{GameState, UpdateSettings},
    collisions::{update_rock_collisions, update_submarine_collisions},
    objects::{interact_with_object, update_objects, Object, ObjectType},
    sonar::update_sonar,
    wires::WireColor,
};

/// A request to mutate state. Created by the UI and player actions.
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
}

pub(crate) enum CellCommand {
    EditWires { color: WireColor },
    EditWalls { add: bool },
    EditWater { add: bool },
    AddObject { object_type: ObjectType },
}

pub(crate) enum UpdateEvent {
    Submarine { submarine_id: usize, submarine_event: SubmarineUpdateEvent },
}

pub(crate) enum SubmarineUpdateEvent {
    SonarUpdated,
    WallsUpdated,
    WiresUpdated,
    SignalsUpdated,
}

pub(crate) fn update_game(
    commands: &[Command],
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
                if let Some(submarine) = game_state.submarines.get_mut(*submarine_id) {
                    if let Some(object) = submarine.objects.get_mut(*object_id) {
                        interact_with_object(object);
                    }
                };
            }
            Command::Cell {
                submarine_id,
                cell,
                cell_command,
            } => {
                if let Some(submarine) = game_state.submarines.get_mut(*submarine_id) {
                    let water_cell = submarine.water_grid.cell_mut(cell.0, cell.1);

                    match cell_command {
                        CellCommand::EditWater { add: true } => water_cell.fill(),
                        CellCommand::EditWater { add: false } => water_cell.empty(),
                        CellCommand::EditWalls { add: true } => water_cell.make_wall(),
                        CellCommand::EditWalls { add: false } => water_cell.clear_wall(),
                        CellCommand::EditWires { color } => {
                            submarine.wire_grid.make_wire(cell.0, cell.1, *color)
                        }
                        CellCommand::AddObject { object_type } => {
                            submarine.objects.push(Object {
                                object_type: object_type.clone(),
                                position: (cell.0 as u32, cell.1 as u32),
                                current_frame: 0,
                            });
                        }
                    }

                    match cell_command {
                        CellCommand::EditWater { .. } | CellCommand::EditWalls { .. } => {
                            events.push(UpdateEvent::Submarine {
                                submarine_id: *submarine_id,
                                submarine_event: SubmarineUpdateEvent::WallsUpdated,
                            });
                        }
                        CellCommand::EditWires { .. } => {
                            events.push(UpdateEvent::Submarine {
                                submarine_id: *submarine_id,
                                submarine_event: SubmarineUpdateEvent::WiresUpdated,
                            });
                        }
                        CellCommand::AddObject { .. } => (),
                    }
                }
            }
            Command::ClearWater { submarine_id } => {
                if let Some(submarine) = game_state.submarines.get_mut(*submarine_id) {
                    submarine.water_grid.clear();
                }
            }
            Command::ChangeUpdateSettings { update_settings } => {
                game_state.update_settings = update_settings.clone()
            }
        }
    }

    let update_settings = &game_state.update_settings;

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
                submarine
                    .wire_grid
                    .update(&mut signals_updated);

                if signals_updated {
                    events.push(UpdateEvent::Submarine {
                        submarine_id: sub_index,
                        submarine_event: SubmarineUpdateEvent::SignalsUpdated,
                    });
                }
            }
        }
        if update_settings.update_objects {
            let mut walls_updated = false;
            update_objects(submarine, &mut walls_updated);

            if walls_updated {
                events.push(UpdateEvent::Submarine {
                    submarine_id: sub_index,
                    submarine_event: SubmarineUpdateEvent::WallsUpdated,
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
                    submarine_event: SubmarineUpdateEvent::SonarUpdated,
                });
            }
        }

        if update_settings.update_collision {
            game_state.collisions.clear();
            update_rock_collisions(submarine, &game_state.rock_grid, &mut game_state.collisions);
        }
    }

    for submarine in &mut game_state.submarines {
        submarine.collisions.clear();
    }

    if update_settings.update_collision {
        for sub1_index in 0..game_state.submarines.len() {
            for sub2_index in sub1_index+1..game_state.submarines.len() {
                let (left, right) = game_state.submarines.split_at_mut(sub2_index);
                let submarine1 = &mut left[sub1_index];
                let submarine2 = &mut right[0];

                update_submarine_collisions(submarine1, submarine2);
                update_submarine_collisions(submarine2, submarine1);
            }
        }
    }
}
