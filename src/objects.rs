use serde::{Deserialize, Serialize};

use crate::{app::{Navigation, SubmarineState}, resources::MutableSubResources};

#[derive(Serialize, Deserialize)]
pub(crate) struct Object {
    pub object_type: ObjectType,

    pub position: (u32, u32),

    #[serde(default, skip_serializing_if = "is_default")]
    pub current_frame: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum ObjectType {
    Door {
        #[serde(default, skip_serializing_if = "is_default")]
        state: DoorState,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
    },
    VerticalDoor {
        #[serde(default, skip_serializing_if = "is_default")]
        state: DoorState,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
    },
    Reactor {
        active: bool,
    },
    Lamp,
    Gauge {
        #[serde(default, skip_serializing_if = "is_default")]
        value: i8,
    },
    SmallPump {
        #[serde(default, skip_serializing_if = "is_default")]
        target_speed: i8,
        #[serde(default, skip_serializing_if = "is_default")]
        speed: i8,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
    },
    LargePump {
        #[serde(default, skip_serializing_if = "is_default")]
        target_speed: i8,
        #[serde(default, skip_serializing_if = "is_default")]
        speed: i8,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
    },
    JunctionBox,
    NavController {
        active: bool,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
    },
    Sonar {
        active: bool,
        #[serde(default, skip_serializing_if = "is_default")]
        powered: bool,
        #[serde(default, skip_serializing)]
        sonar_info: SonarInfo,
    },
    Engine {
        #[serde(default, skip_serializing_if = "is_default")]
        target_speed: i8,
        #[serde(default, skip_serializing_if = "is_default")]
        speed: i8,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) enum DoorState {
    Opening,
    Closing,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub(crate) struct SonarInfo {
    pub cursor: Option<(f32, f32)>,
    pub set_target: Option<(f32, f32)>,
}

pub(crate) struct NavControl {
    pub target_speed: (i32, i32),
    pub target_acceleration: (i32, i32),
    pub engine_and_pump_speed: (i32, i32),
}

fn is_default<T: Default + Eq>(value: &T) -> bool {
    *value == T::default()
}

impl Default for DoorState {
    fn default() -> Self {
        DoorState::Closing
    }
}

impl Object {
    pub(crate) fn active_sonar_info(&self) -> Option<&SonarInfo> {
        if let ObjectType::Sonar {
            active: true,
            powered: true,
            sonar_info,
        } = &self.object_type
        {
            Some(sonar_info)
        } else {
            None
        }
    }
}

pub(crate) const OBJECT_TYPES: &'static [(&'static str, ObjectType)] = &[
    (
        "Hatch",
        ObjectType::Door {
            state: DoorState::Closing,
            progress: 0,
        },
    ),
    (
        "Door",
        ObjectType::VerticalDoor {
            state: DoorState::Closing,
            progress: 0,
        },
    ),
    ("Reactor", ObjectType::Reactor { active: false }),
    ("Lamp", ObjectType::Lamp),
    ("Gauge", ObjectType::Gauge { value: 0 }),
    (
        "Small pump",
        ObjectType::SmallPump {
            target_speed: 0,
            speed: 0,
            progress: 0,
        },
    ),
    (
        "Large pump",
        ObjectType::LargePump {
            target_speed: 0,
            speed: 0,
            progress: 0,
        },
    ),
    ("Junction box", ObjectType::JunctionBox),
    (
        "Nav controller",
        ObjectType::NavController {
            active: true,
            progress: 0,
        },
    ),
    (
        "Sonar",
        ObjectType::Sonar {
            active: true,
            powered: false,
            sonar_info: SonarInfo {
                cursor: None,
                set_target: None,
            },
        },
    ),
    (
        "Engine",
        ObjectType::Engine {
            target_speed: 0,
            speed: 0,
            progress: 0,
        },
    ),
];

// What an object does on every physics update tick.
pub(crate) fn update_objects(submarine: &mut SubmarineState, mutable_resources: &mut MutableSubResources) {
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
                        let cell_x = object.position.0 + x;
                        let cell_y = object.position.1 + y;

                        let cell = water_grid.cell_mut(cell_x as usize, cell_y as usize);

                        if should_be_open(x) {
                            if !cell.is_inside() {
                                cell.make_inside();
                                mutable_resources.walls_updated = true;
                            }
                        } else if !cell.is_wall() {
                            cell.make_wall();
                            mutable_resources.walls_updated = true;
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

                    let cell_x = object.position.0 + x;
                    let cell_y = object.position.1 + y;

                    let cell = water_grid.cell_mut(cell_x as usize, cell_y as usize);

                    if should_be_open(y) {
                        if !cell.is_inside() {
                            cell.make_inside();
                        }
                    } else if !cell.is_wall() {
                        cell.make_wall();
                    }
                }
            }
            ObjectType::Reactor { active } => {
                let cell_x = object.position.0 + 29;
                let cell_y = object.position.1 + 5;

                let cell = wire_grid.cell_mut(cell_x as usize, cell_y as usize);

                if *active {
                    object.current_frame = 0;
                    cell.send_power(200);
                } else {
                    object.current_frame = 1;
                }
            }
            ObjectType::Lamp => {
                let cell_x = object.position.0 + 3;
                let cell_y = object.position.1 + 1;

                let cell = wire_grid.cell(cell_x as usize, cell_y as usize);

                object.current_frame = 0;

                if cell.minimum_power(10) {
                    object.current_frame = 1;
                }
            }
            ObjectType::Gauge { value } => {
                let cell_x = object.position.0 + 4;
                let cell_y = object.position.1 + 1;

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
            ObjectType::SmallPump {
                target_speed,
                speed,
                progress,
            } => {
                let cell_x = object.position.0 + 3;
                let cell_y = object.position.1 + 2;

                let cell = wire_grid.cell(cell_x as usize + 2, cell_y as usize);
                if let Some(logic_value) = cell.receive_logic() {
                    *target_speed = logic_value;
                }
                let cell = wire_grid.cell(cell_x as usize, cell_y as usize);
                let target_speed = if cell.minimum_power(50) {
                    *target_speed
                } else {
                    0
                };

                *speed = ((*speed as i16 * 9 + target_speed as i16) / 10) as i8;

                if *speed >= 0 {
                    *progress = progress.wrapping_add((*speed / 4) as u8);
                } else {
                    *progress = progress.wrapping_sub((speed.abs() / 4) as u8);
                }

                object.current_frame = (*progress as u8 / (u8::MAX / 4)).clamp(0, 3) as u16;

                let cell_x = object.position.0 + 7;
                let cell_y = object.position.1 + 5;

                let cell = water_grid.cell_mut(cell_x as usize, cell_y as usize);

                cell.add_level(*speed as i32 * 2);
            }
            ObjectType::LargePump {
                target_speed,
                speed,
                progress,
            } => {
                let cell_x = object.position.0 + 10;
                let cell_y = object.position.1 + 2;

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

                *speed = ((*speed as i16 * 9 + target_speed as i16) / 10) as i8;

                if *speed >= 0 {
                    *progress = progress.wrapping_add((*speed / 4) as u8);
                } else {
                    *progress = progress.wrapping_sub((speed.abs() / 4) as u8);
                }

                object.current_frame = (*progress as u8 / (u8::MAX / 4)).clamp(0, 3) as u16;

                for y in 0..4 {
                    for x in 0..4 {
                        let cell_x = object.position.0 + 23 + x;
                        let cell_y = object.position.1 + 12 + y;

                        let cell = water_grid.cell_mut(cell_x as usize, cell_y as usize);

                        cell.add_level(*speed as i32 * 2);
                    }
                }
            }
            ObjectType::JunctionBox => {
                let cell_x = object.position.0 as usize + 3;
                let cell_y = object.position.1 as usize + 1;

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
                let cell_x = object.position.0 as usize + 2;
                let cell_y = object.position.1 as usize + 4;

                let nav_control = compute_navigation(&submarine.navigation);
                object.current_frame = 0;
                let cell = wire_grid.cell(cell_x, cell_y);
                if *active && cell.minimum_power(50) {
                    let (engine_speed, pump_speed) = nav_control.engine_and_pump_speed;

                    wire_grid
                        .cell_mut(cell_x + 6, cell_y + 2)
                        .send_logic(engine_speed.clamp(i8::MIN.into(), i8::MAX.into()) as i8);

                    wire_grid
                        .cell_mut(cell_x + 6, cell_y)
                        .send_logic(pump_speed.clamp(i8::MIN.into(), i8::MAX.into()) as i8);

                    *progress = (*progress + 1) % (8 * 5);

                    object.current_frame = (*progress as u16 / 8) % 5 + 1;
                }
            }
            ObjectType::Sonar {
                active,
                powered,
                sonar_info,
            } => {
                let x = object.position.0 as usize + 2;
                let y = object.position.1 as usize + 15;

                *powered = wire_grid.cell(x, y).minimum_power(100);

                if *powered && *active {
                    if let Some(target) = sonar_info.set_target {
                        // 16 sub-cells per rock-cell, 16 movement points per rock-cell
                        let world_ratio = 16.0 * 16.0;
                        // 75 rock-cells radius, on 6-pixels per cell resolution
                        let sonar_ratio = 75.0 / 6.0;

                        let target_x = submarine.navigation.position.0
                            + (target.0 * world_ratio * sonar_ratio) as i32;
                        let target_y = submarine.navigation.position.1
                            + (target.1 * world_ratio * sonar_ratio) as i32;
                        submarine.navigation.target = (target_x, target_y);

                        sonar_info.set_target = None;
                    }
                }

                object.current_frame = if *powered && *active { 0 } else { 1 };
            }
            ObjectType::Engine {
                target_speed,
                speed,
                progress,
            } => {
                let cell_x = object.position.0 + 36;
                let cell_y = object.position.1 + 6;

                let cell = wire_grid.cell(cell_x as usize, cell_y as usize + 2);
                if let Some(logic_value) = cell.receive_logic() {
                    *target_speed = logic_value;
                }
                let cell = wire_grid.cell(cell_x as usize, cell_y as usize);
                let target_speed = if cell.minimum_power(100) {
                    *target_speed
                } else {
                    0
                };

                *speed = ((*speed as i16 * 9 + target_speed as i16) / 10) as i8;

                if *speed >= 0 {
                    *progress = progress.wrapping_add((*speed / 4) as u8);
                } else {
                    *progress = progress.wrapping_sub((speed.abs() / 4) as u8);
                }

                let frames = 24;
                object.current_frame =
                    (*progress as u8 / (u8::MAX / frames)).clamp(0, frames - 1) as u16;

                submarine.navigation.acceleration.0 = match *speed {
                    -128..=-96 => -4,
                    -95..=-64 => -3,
                    -63..=-32 => -2,
                    -31..=-16 => -1,
                    -15..=15 => 0,
                    16..=31 => 1,
                    32..=63 => 2,
                    64..=95 => 3,
                    96..=127 => 4,
                };
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
        ObjectType::SmallPump { target_speed, .. } => cycle_i8(target_speed),
        ObjectType::LargePump { target_speed, .. } => cycle_i8(target_speed),
        ObjectType::JunctionBox => (),
        ObjectType::NavController { active, .. } => *active = !*active,
        ObjectType::Sonar {
            active, sonar_info, ..
        } => {
            if let Some(cursor) = sonar_info.cursor {
                sonar_info.set_target = Some(cursor);
            } else {
                *active = !*active;
            }
        }
        ObjectType::Engine { target_speed, .. } => cycle_i8(target_speed),
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

pub(crate) fn hover_over_object(object: &mut Object, hover_position: (f32, f32)) {
    if let ObjectType::Sonar {
        active: true,
        sonar_info,
        ..
    } = &mut object.object_type
    {
        let sonar_middle = (9.5, 7.5);
        let cursor = (
            hover_position.0 - sonar_middle.0,
            hover_position.1 - sonar_middle.1,
        );

        let length_squared = cursor.0 * cursor.0 + cursor.1 * cursor.1;
        sonar_info.cursor = if length_squared < 5.0 * 5.0 {
            Some(cursor)
        } else {
            None
        };
    }
}

pub(crate) fn compute_navigation(navigation: &Navigation) -> NavControl {
    // X axis - control engine
    let target_speed_x = ((navigation.target.0 - navigation.position.0) / 4).clamp(-2048, 2048);

    let target_acceleration_x = ((target_speed_x - navigation.speed.0) / 256).clamp(-4, 4);
    let engine_speed = 32 * target_acceleration_x;

    // Y axis - control pumps in ballast tanks
    let target_speed_y = ((navigation.target.1 - navigation.position.1) / 4).clamp(-2048, 2048);
    let target_acceleration_y = ((target_speed_y - navigation.speed.1) / 256).clamp(-3, 3);
    let pump_speed = 32 * (target_acceleration_y - navigation.acceleration.1).clamp(-4, 4);

    NavControl {
        target_speed: (target_speed_x, target_speed_y),
        target_acceleration: (target_acceleration_x, target_acceleration_y),
        engine_and_pump_speed: (engine_speed, pump_speed),
    }
}
