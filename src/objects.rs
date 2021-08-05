use crate::app::SubmarineState;

pub(crate) struct Object {
    pub object_type: ObjectType,
    pub position_x: u32,
    pub position_y: u32,
    pub current_frame: u16,
    pub frames: u16,
}

pub(crate) enum ObjectType {
    Door {
        state: DoorState,
        progress: u8,
    },
    VerticalDoor {
        state: DoorState,
        progress: u8,
    },
    Reactor {
        active: bool,
    },
    Lamp,
    Gauge {
        value: i8,
    },
    // SmallPump,
    LargePump {
        target_speed: i8,
        speed: i8,
        progress: u8,
    },
    JunctionBox,
    NavController {
        active: bool,
        progress: u8,
    },
    Sonar {
        active: bool,
        powered: bool,
    },
}

pub(crate) enum DoorState {
    Opening,
    Closing,
}

impl Object {
    pub(crate) fn size(&self) -> (u32, u32) {
        match self.object_type {
            ObjectType::Door { .. } => (22, 5),
            ObjectType::VerticalDoor { .. } => (5, 17),
            ObjectType::Reactor { .. } => (32, 17),
            ObjectType::Lamp => (5, 4),
            ObjectType::Gauge { .. } => (7, 7),
            ObjectType::LargePump { .. } => (30, 18),
            ObjectType::JunctionBox => (6, 8),
            ObjectType::NavController { .. } => (9, 15),
            ObjectType::Sonar { .. } => (19, 17),
        }
    }

    pub(crate) fn is_active_sonar(&self) -> bool {
        matches!(
            self.object_type,
            ObjectType::Sonar {
                active: true,
                powered: true
            }
        )
    }
}

// What an object does on every physics update tick.
pub(crate) fn update_objects(submarine: &mut SubmarineState) {
    let SubmarineState {
        objects,
        water_grid,
        wire_grid,
        ..
    } = submarine;

    for object in objects {
        match &mut object.object_type {
            ObjectType::Door { state, progress } => {
                match state {
                    DoorState::Opening => *progress = (*progress + 1).min(15),
                    DoorState::Closing => *progress = progress.saturating_sub(1),
                }
                object.current_frame = (*progress as u16 * 8 / 15).clamp(0, 7);

                let open_cells = match *progress {
                    x if (0..3).contains(&x) => (12..12),
                    x if (3..5).contains(&x) => (11..13),
                    x if (5..7).contains(&x) => (10..14),
                    x if (7..9).contains(&x) => (9..15),
                    x if (9..11).contains(&x) => (8..16),
                    x if (11..13).contains(&x) => (7..17),
                    _ => (6..18),
                };

                let should_be_open = |x: u32| open_cells.contains(&x);

                for y in 2..5 {
                    for x in 6..19 {
                        let cell_x = object.position_x + x;
                        let cell_y = object.position_y + y;

                        let cell = water_grid.cell_mut(cell_x as usize, cell_y as usize);

                        if should_be_open(x) {
                            if !cell.is_inside() {
                                cell.make_inside();
                            }
                        } else {
                            if !cell.is_wall() {
                                cell.make_wall();
                            }
                        }
                    }
                }
            }
            ObjectType::VerticalDoor { state, progress } => {
                match state {
                    DoorState::Opening => *progress = (*progress + 1).min(15),
                    DoorState::Closing => *progress = progress.saturating_sub(1),
                }
                object.current_frame = (*progress as u16 * 9 / 15).clamp(0, 8);

                let open_cells = match *progress {
                    0 => 0,
                    x if (1..3).contains(&x) => 1,
                    x if (3..5).contains(&x) => 2,
                    x if (5..7).contains(&x) => 4,
                    x if (7..9).contains(&x) => 6,
                    x if (9..11).contains(&x) => 9,
                    x if (11..13).contains(&x) => 10,
                    _ => 12,
                };

                let should_be_open = |y: u32| 17 - y <= open_cells;

                for y in 5..17 {
                    let x = 3;

                    let cell_x = object.position_x + x;
                    let cell_y = object.position_y + y;

                    let cell = water_grid.cell_mut(cell_x as usize, cell_y as usize);

                    if should_be_open(y) {
                        if !cell.is_inside() {
                            cell.make_inside();
                        }
                    } else {
                        if !cell.is_wall() {
                            cell.make_wall();
                        }
                    }
                }
            }
            ObjectType::Reactor { active } => {
                let cell_x = object.position_x + 29;
                let cell_y = object.position_y + 5;

                let cell = wire_grid.cell_mut(cell_x as usize, cell_y as usize);

                if *active {
                    object.current_frame = 0;
                    cell.send_power(200);
                } else {
                    object.current_frame = 1;
                }
            }
            ObjectType::Lamp => {
                let cell_x = object.position_x + 3;
                let cell_y = object.position_y + 1;

                let cell = wire_grid.cell(cell_x as usize, cell_y as usize);

                object.current_frame = 0;

                if cell.minimum_power(10) {
                    object.current_frame = 1;
                }
            }
            ObjectType::Gauge { value } => {
                let cell_x = object.position_x + 4;
                let cell_y = object.position_y + 1;

                let cell = wire_grid.cell(cell_x as usize, cell_y as usize);
                if let Some(logic_value) = cell.receive_logic() {
                    *value = logic_value;
                }
                let cell = wire_grid.cell_mut(cell_x as usize, cell_y as usize + 5);
                cell.send_logic(*value);

                object.current_frame = match *value {
                    -128..=-96 => 0,
                    -95..=-32 => 1,
                    -31..=31 => 2,
                    32..=95 => 3,
                    96..=127 => 4,
                };
            }
            ObjectType::LargePump {
                target_speed,
                speed,
                progress,
            } => {
                let cell_x = object.position_x + 10;
                let cell_y = object.position_y + 2;

                let cell = wire_grid.cell(cell_x as usize + 3, cell_y as usize);
                if let Some(logic_value) = cell.receive_logic() {
                    *target_speed = logic_value;
                }
                let cell = wire_grid.cell(cell_x as usize, cell_y as usize);
                let target_speed = if cell.minimum_power(100) {
                    *target_speed
                } else {
                    0
                };

                *speed = ((*speed as i16 + target_speed as i16) / 2) as i8;

                if *speed >= 0 {
                    *progress = progress.wrapping_add((*speed / 4) as u8);
                } else {
                    *progress = progress.wrapping_sub((speed.abs() / 4) as u8);
                }

                object.current_frame = (*progress as u8 / (u8::MAX / 4)).clamp(0, 3) as u16;

                for y in 0..4 {
                    for x in 0..4 {
                        let cell_x = object.position_x + 23 + x;
                        let cell_y = object.position_y + 12 + y;

                        let cell = water_grid.cell_mut(cell_x as usize, cell_y as usize);

                        cell.add_level(*speed as i32 * 2);
                    }
                }
            }
            ObjectType::JunctionBox => {
                let cell_x = object.position_x as usize + 3;
                let cell_y = object.position_y as usize + 1;

                let outputs = &[(3, 2), (3, 3), (3, 4), (3, 5)];

                let cell = wire_grid.cell(cell_x, cell_y);
                if let Some(logic_value) = cell.receive_logic() {
                    for output in outputs {
                        wire_grid
                            .cell_mut(cell_x + output.0, cell_y + output.1)
                            .send_logic(logic_value);
                    }
                }

                let cell = wire_grid.cell(cell_x, cell_y);
                if let Some(power_value) = cell.receive_power() {
                    for output in outputs {
                        wire_grid
                            .cell_mut(cell_x + output.0, cell_y + output.1)
                            .send_power(power_value);
                    }
                }
            }
            ObjectType::NavController { active, progress } => {
                let cell_x = object.position_x as usize + 2;
                let cell_y = object.position_y as usize + 4;

                object.current_frame = 0;
                let cell = wire_grid.cell(cell_x, cell_y);
                if *active && cell.minimum_power(50) {
                    let target_speed = 0;

                    let target_acceleration = match submarine.speed.1 - target_speed {
                        (i32::MIN..=-255) => 2,
                        (-256..=-15) => 1,
                        (-16..=15) => 0,
                        (16..=255) => -1,
                        (256..=i32::MAX) => -2,
                    };

                    let pump_speed = match target_acceleration - submarine.acceleration.1 {
                        (i32::MIN..=-4) => -3,
                        (-3..=-2) => -2,
                        -1 => -1,
                        0 => 0,
                        1 => 1,
                        (2..=3) => 2,
                        (4..=i32::MAX) => 3,
                    };

                    wire_grid
                        .cell_mut(cell_x + 7, cell_y)
                        .send_logic(pump_speed * 32);

                    *progress = (*progress + 1) % (8 * 5);

                    object.current_frame = (*progress as u16 / 8) % 5 + 1;
                }
            }
            ObjectType::Sonar { active, powered } => {
                let x = object.position_x as usize + 2;
                let y = object.position_y as usize + 15;

                *powered = wire_grid.cell(x, y).minimum_power(100);

                object.current_frame = if *powered && *active { 0 } else { 1 };
            }
        }
    }
}

// What an object does when left-clicked.
pub(crate) fn interact_with_object(object: &mut Object) {
    match &mut object.object_type {
        ObjectType::Door { state, .. } | ObjectType::VerticalDoor { state, .. } => {
            *state = match state {
                DoorState::Opening => DoorState::Closing,
                DoorState::Closing => DoorState::Opening,
            }
        }
        ObjectType::Reactor { active } => *active = !*active,
        ObjectType::Lamp => (),
        ObjectType::Gauge { value } => cycle_i8(value),
        ObjectType::LargePump { target_speed, .. } => cycle_i8(target_speed),
        ObjectType::JunctionBox => (),
        ObjectType::NavController { active, .. } => *active = !*active,
        ObjectType::Sonar { active, .. } => *active = !*active,
    }
}

fn cycle_i8(value: &mut i8) {
    *value = match *value {
        0 => 64,
        64 => 127,
        127 => -128,
        -128 => -64,
        -64 => 0,
        _ => 0,
    };
}
