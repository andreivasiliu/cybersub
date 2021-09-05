use serde::{Deserialize, Serialize};

use crate::game_state::state::{Navigation, SubmarineState};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Object {
    pub object_type: ObjectType,

    pub position: (u32, u32),

    pub powered: bool,
}

#[derive(Serialize, Deserialize, Clone)]
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
    SmallPump {
        target_speed: i8,
        speed: i8,
        progress: u8,
    },
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
        navigation_target: Option<(usize, usize)>,
    },
    Engine {
        target_speed: i8,
        speed: i8,
        progress: u8,
    },
    Battery {
        charge: u16,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ObjectTemplate {
    pub object_type: ObjectTypeTemplate,
    pub position: (u32, u32),
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum ObjectTypeTemplate {
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
        navigation_target: Option<(usize, usize)>,
    },
    Engine {
        #[serde(default, skip_serializing_if = "is_default")]
        target_speed: i8,
        #[serde(default, skip_serializing_if = "is_default")]
        speed: i8,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
    },
    Battery {
        charge: u16,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) enum DoorState {
    Opening,
    Closing,
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
    pub(crate) fn active_sonar_target(&self) -> Option<Option<(usize, usize)>> {
        if self.powered {
            if let ObjectType::Sonar {
                active: true,
                navigation_target,
            } = &self.object_type
            {
                Some(*navigation_target)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub(crate) const OBJECT_TYPES: &[(&str, ObjectType)] = &[
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
            navigation_target: None,
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
    ("Battery", ObjectType::Battery { charge: 300 }),
];

// What an object does on every physics update tick.
pub(crate) fn update_objects(submarine: &mut SubmarineState, walls_updated: &mut bool) {
    let SubmarineState {
        objects,
        water_grid,
        wire_grid,
        ..
    } = submarine;

    for object in objects {
        let powered = &mut object.powered;

        match &mut object.object_type {
            ObjectType::Door { state, progress } => {
                match state {
                    DoorState::Opening => *progress = (*progress + 1).min(15),
                    DoorState::Closing => *progress = progress.saturating_sub(1),
                }

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
                                *walls_updated = true;
                            }
                        } else if !cell.is_wall() {
                            cell.make_wall();
                            *walls_updated = true;
                        }
                    }
                }
            }
            ObjectType::VerticalDoor { state, progress } => {
                match state {
                    DoorState::Opening => *progress = (*progress + 1).min(15),
                    DoorState::Closing => *progress = progress.saturating_sub(1),
                }

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
                            *walls_updated = true;
                        }
                    } else if !cell.is_wall() {
                        cell.make_wall();
                        *walls_updated = true;
                    }
                }
            }
            ObjectType::Reactor { active } => {
                let cell_x = object.position.0 + 29;
                let cell_y = object.position.1 + 5;

                let cell = wire_grid.cell_mut(cell_x as usize, cell_y as usize);

                if *active {
                    cell.send_power(200);
                }
            }
            ObjectType::Lamp => {
                let cell_x = object.position.0 + 3;
                let cell_y = object.position.1 + 1;

                let cell = wire_grid.cell(cell_x as usize, cell_y as usize);

                *powered = cell.minimum_power(10);
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

                let cell_x = object.position.0 + 7;
                let cell_y = object.position.1 + 5;

                let cell = water_grid.cell_mut(cell_x as usize, cell_y as usize);

                cell.add_level(*speed as i32 * 3);
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
                let cell = wire_grid.cell(cell_x, cell_y);
                object.powered = false;
                if *active && cell.minimum_power(50) {
                    let (engine_speed, pump_speed) = nav_control.engine_and_pump_speed;

                    wire_grid
                        .cell_mut(cell_x + 6, cell_y + 2)
                        .send_logic(engine_speed.clamp(i8::MIN.into(), i8::MAX.into()) as i8);

                    wire_grid
                        .cell_mut(cell_x + 6, cell_y)
                        .send_logic(pump_speed.clamp(i8::MIN.into(), i8::MAX.into()) as i8);

                    *progress = (*progress + 1) % (8 * 5);

                    object.powered = true;
                }
            }
            ObjectType::Sonar {
                active,
                navigation_target,
            } => {
                let x = object.position.0 as usize + 2;
                let y = object.position.1 as usize + 15;

                *powered = wire_grid.cell(x, y).minimum_power(100);

                if *powered && *active {
                    if let Some(target) = *navigation_target {
                        submarine.navigation.target = (target.0 as i32, target.1 as i32);
                    }
                }
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
            ObjectType::Battery { charge } => {
                let cell_x = object.position.0 as usize + 2;
                let cell_y = object.position.1 as usize + 4;

                let cell = wire_grid.cell(cell_x, cell_y);
                if cell.minimum_power(100) {
                    // 3 minutes: 3m * 60s * 30ups
                    *charge = (*charge + 2).min(5400);
                }

                if *charge > 0 {
                    *charge -= 1;

                    wire_grid.cell_mut(cell_x + 5, cell_y).send_power(100);
                }
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
        ObjectType::Lamp { .. } => (),
        ObjectType::Gauge { value } => cycle_i8(value),
        ObjectType::SmallPump { target_speed, .. } => cycle_i8(target_speed),
        ObjectType::LargePump { target_speed, .. } => cycle_i8(target_speed),
        ObjectType::JunctionBox => (),
        ObjectType::NavController { active, .. } => *active = !*active,
        ObjectType::Sonar { active, .. } => *active = !*active,
        ObjectType::Engine { target_speed, .. } => cycle_i8(target_speed),
        ObjectType::Battery { .. } => (),
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

pub(crate) fn current_frame(object: &Object) -> (u16, u16) {
    let current_frame;
    let powered = &object.powered;

    match &object.object_type {
        ObjectType::Door { progress, .. } => {
            current_frame = (*progress as u16 * 8 / 15).clamp(0, 7);
        },
        ObjectType::VerticalDoor { progress, .. } => {
            current_frame = (*progress as u16 * 9 / 15).clamp(0, 8);
        },
        ObjectType::Reactor { active } => {
            if *active {
                current_frame = 0;
            } else {
                current_frame = 1;
            }
        },
        ObjectType::Lamp => {
            if *powered {
                current_frame = 1;
            } else {
                current_frame = 0;
            }
        },
        ObjectType::Gauge { value } => {
            current_frame = match *value {
                -128..=-96 => 0,
                -95..=-32 => 1,
                -31..=31 => 2,
                32..=95 => 3,
                96..=127 => 4,
            };
        },
        ObjectType::SmallPump { progress, .. } => {
            current_frame = (*progress as u8 / (u8::MAX / 4)).clamp(0, 3) as u16;
        },
        ObjectType::LargePump { progress, .. } => {
            current_frame = (*progress as u8 / (u8::MAX / 4)).clamp(0, 3) as u16;
        },
        ObjectType::JunctionBox => {
            current_frame = 0;
        },
        ObjectType::NavController { active, progress, .. } => {
            current_frame = if *active && *powered {
                (*progress as u16 / 8) % 5 + 1
            } else {
                0
            };
        },
        ObjectType::Sonar { active, .. } => {
            current_frame = if *powered && *active { 0 } else { 1 };
        },
        ObjectType::Engine { progress, .. } => {
            let frames = 24;
            current_frame =
                (*progress as u8 / (u8::MAX / frames)).clamp(0, frames - 1) as u16;

        },
        ObjectType::Battery { charge } => {
            current_frame = if *charge == 0 {
                7
            } else {
                7 - (*charge * 8 / 5400).clamp(1, 7)
            };
        },
    }

    (current_frame, 0)
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

impl ObjectTemplate {
    pub fn from_object(object: &Object) -> Self {
        let object_type = match object.object_type.clone() {
            ObjectType::Door { state, progress } => ObjectTypeTemplate::Door { state, progress },
            ObjectType::VerticalDoor { state, progress } => {
                ObjectTypeTemplate::VerticalDoor { state, progress }
            }
            ObjectType::Reactor { active } => ObjectTypeTemplate::Reactor { active },
            ObjectType::Lamp { .. }=> ObjectTypeTemplate::Lamp,
            ObjectType::Gauge { value } => ObjectTypeTemplate::Gauge { value },
            ObjectType::SmallPump {
                target_speed,
                speed,
                progress,
            } => ObjectTypeTemplate::SmallPump {
                target_speed,
                speed,
                progress,
            },
            ObjectType::LargePump {
                target_speed,
                speed,
                progress,
            } => ObjectTypeTemplate::LargePump {
                target_speed,
                speed,
                progress,
            },
            ObjectType::JunctionBox => ObjectTypeTemplate::JunctionBox,
            ObjectType::NavController { active, progress } => {
                ObjectTypeTemplate::NavController { active, progress }
            }
            ObjectType::Sonar {
                active,
                navigation_target,
            } => ObjectTypeTemplate::Sonar {
                active,
                navigation_target,
            },
            ObjectType::Engine {
                target_speed,
                speed,
                progress,
            } => ObjectTypeTemplate::Engine {
                target_speed,
                speed,
                progress,
            },
            ObjectType::Battery { charge } => ObjectTypeTemplate::Battery { charge },
        };

        ObjectTemplate {
            object_type,
            position: object.position,
        }
    }

    pub fn to_object(&self) -> Object {
        let object_type = match self.object_type.clone() {
            ObjectTypeTemplate::Door { state, progress } => ObjectType::Door { state, progress },
            ObjectTypeTemplate::VerticalDoor { state, progress } => {
                ObjectType::VerticalDoor { state, progress }
            }
            ObjectTypeTemplate::Reactor { active } => ObjectType::Reactor { active },
            ObjectTypeTemplate::Lamp => ObjectType::Lamp,
            ObjectTypeTemplate::Gauge { value } => ObjectType::Gauge { value },
            ObjectTypeTemplate::SmallPump {
                target_speed,
                speed,
                progress,
            } => ObjectType::SmallPump {
                target_speed,
                speed,
                progress,
            },
            ObjectTypeTemplate::LargePump {
                target_speed,
                speed,
                progress,
            } => ObjectType::LargePump {
                target_speed,
                speed,
                progress,
            },
            ObjectTypeTemplate::JunctionBox => ObjectType::JunctionBox,
            ObjectTypeTemplate::NavController { active, progress } => {
                ObjectType::NavController { active, progress }
            }
            ObjectTypeTemplate::Sonar {
                active,
                navigation_target,
            } => ObjectType::Sonar {
                active,
                navigation_target,
            },
            ObjectTypeTemplate::Engine {
                target_speed,
                speed,
                progress,
            } => ObjectType::Engine {
                target_speed,
                speed,
                progress,
            },
            ObjectTypeTemplate::Battery { charge } => ObjectType::Battery { charge },
        };

        Object {
            object_type,
            position: self.position,
            powered: false,
        }
    }
}
