use serde::{Deserialize, Serialize};

use crate::game_state::state::{Navigation, SubmarineState};

use super::wires::{StoredSignal, THIN_COLORS};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Object {
    pub object_type: ObjectType,

    pub position: (u32, u32),

    pub powered: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    JunctionBox {
        enabled: bool,
        progress: u8,
    },
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
    BundleInput {
        sub_bundle: u8,
    },
    BundleOutput {
        sub_bundle: u8,
    },
    DockingConnectorTop {
        state: DoorState,
        progress: u8,
        connected: bool,
        previous_connected: bool,
    },
    DockingConnectorBottom {
        state: DoorState,
        progress: u8,
        connected: bool,
        previous_connected: bool,
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
    JunctionBox {
        enabled: bool,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
    },
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
    BundleInput {
        sub_bundle: u8,
    },
    BundleOutput {
        sub_bundle: u8,
    },
    DockingConnectorTop {
        #[serde(default, skip_serializing_if = "is_default")]
        state: DoorState,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
        #[serde(default, skip_serializing_if = "is_default")]
        connected: bool,
        #[serde(default, skip_serializing_if = "is_default")]
        previous_connected: bool,
    },
    DockingConnectorBottom {
        #[serde(default, skip_serializing_if = "is_default")]
        state: DoorState,
        #[serde(default, skip_serializing_if = "is_default")]
        progress: u8,
        #[serde(default, skip_serializing_if = "is_default")]
        connected: bool,
        #[serde(default, skip_serializing_if = "is_default")]
        previous_connected: bool,
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

impl DoorState {
    #[must_use = "This method does not mutate the original object."]
    fn toggle(&self) -> DoorState {
        match self {
            DoorState::Opening => DoorState::Closing,
            DoorState::Closing => DoorState::Opening,
        }
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
    (
        "Junction box",
        ObjectType::JunctionBox {
            enabled: false,
            progress: 0,
        },
    ),
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
    ("Bundle input", ObjectType::BundleInput { sub_bundle: 0 }),
    ("Bundle output", ObjectType::BundleOutput { sub_bundle: 0 }),
    (
        "Docking connector (top)",
        ObjectType::DockingConnectorTop {
            state: DoorState::Closing,
            progress: 0,
            connected: false,
            previous_connected: false,
        },
    ),
    (
        "Docking connector (bottom)",
        ObjectType::DockingConnectorBottom {
            state: DoorState::Closing,
            progress: 0,
            connected: false,
            previous_connected: false,
        },
    ),
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
                let cell_x = object.position.0 as usize + 2;
                let cell_y = object.position.1 as usize + 4;

                let logic1 = wire_grid.cell(cell_x, cell_y).receive_logic();
                let logic2 = wire_grid.cell(cell_x + 17, cell_y).receive_logic();

                *powered = false;

                if let Some(logic_value) = logic1.or(logic2) {
                    *state = if logic_value > 0 {
                        DoorState::Opening
                    } else if logic_value < 0 {
                        DoorState::Closing
                    } else {
                        state.toggle()
                    };

                    *powered = true;
                }

                match state {
                    DoorState::Opening => *progress = (*progress + 1).min(15),
                    DoorState::Closing => *progress = progress.saturating_sub(1),
                }

                let open_cells = match *progress {
                    0..=2 => (11..11),
                    3..=4 => (10..12),
                    5..=6 => (9..13),
                    7..=8 => (8..14),
                    9..=10 => (7..15),
                    11..=12 => (6..16),
                    _ => (5..17),
                };

                let should_be_open = |x: u32| open_cells.contains(&x);

                for y in 3..6 {
                    for x in 5..18 {
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
                let cell_y = object.position.1 + 2;

                let cell = wire_grid.cell(cell_x as usize, cell_y as usize);
                if let Some(logic_value) = cell.receive_logic() {
                    *value = logic_value;
                }
                let cell = wire_grid.cell_mut(cell_x as usize, cell_y as usize + 4);
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
                let cell_y = object.position.1 + 3;

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
            ObjectType::JunctionBox { enabled, progress } => {
                let cell_x = object.position.0 as usize + 3;
                let cell_y = object.position.1 as usize + 2;

                let outputs = &[(2, 1), (2, 2), (2, 3), (2, 4)];

                let cell = wire_grid.cell(cell_x, cell_y);
                if let Some(logic_value) = cell.receive_logic() {
                    for output in outputs {
                        wire_grid
                            .cell_mut(cell_x + output.0, cell_y + output.1)
                            .send_logic(logic_value);
                    }
                }

                object.powered = false;
                let cell = wire_grid.cell(cell_x, cell_y);
                if let Some(power_value) = cell.receive_power() {
                    object.powered = true;

                    if *progress >= 15 {
                        for output in outputs {
                            wire_grid
                                .cell_mut(cell_x + output.0, cell_y + output.1)
                                .send_power(power_value);
                        }
                    }
                }

                if *enabled {
                    *progress = (*progress + 1).min(15);
                } else {
                    *progress = progress.saturating_sub(1);
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
            ObjectType::BundleInput { sub_bundle } => {
                let cell_x = object.position.0 as usize + 2;
                let cell_y = object.position.1 as usize + 2;
                let mut wire_bundle = None;

                if let Some(wire_bundle_id) = wire_grid.cell(cell_x, cell_y).bundle_id() {
                    let b2 = wire_grid.cell(cell_x + 1, cell_y).bundle_id();
                    let b3 = wire_grid.cell(cell_x + 2, cell_y).bundle_id();

                    if Some(wire_bundle_id) == b2 && Some(wire_bundle_id) == b3 {
                        let source = *wire_grid.cell(cell_x + 2, cell_y);
                        wire_bundle = wire_grid
                            .wire_bundle_input_mut(wire_bundle_id)
                            .map(|bundle| (source, bundle));
                    }
                }

                if let Some((source, wire_bundle)) = wire_bundle {
                    let sub_bundle: usize = (*sub_bundle).into();
                    let stored_signals = &mut wire_bundle.bundled_cells[sub_bundle];

                    for color in THIN_COLORS {
                        stored_signals[color as usize] = StoredSignal {
                            logic: source.value(color).get_logic(),
                            power: source.value(color).get_power(),
                        };
                    }
                }
            }
            ObjectType::BundleOutput { sub_bundle } => {
                let cell_x = object.position.0 as usize + 2;
                let cell_y = object.position.1 as usize + 2;
                let mut wire_bundle_id = None;

                if let Some(bundle_id) = wire_grid.cell(cell_x, cell_y).bundle_id() {
                    let b2 = wire_grid.cell(cell_x + 1, cell_y).bundle_id();
                    let b3 = wire_grid.cell(cell_x + 2, cell_y).bundle_id();

                    if Some(bundle_id) == b2 && Some(bundle_id) == b3 {
                        wire_bundle_id = Some(bundle_id);
                    }
                }

                let (x, y) = (cell_x + 2, cell_y);

                if let Some(bundle_id) = wire_bundle_id {
                    for color in THIN_COLORS {
                        if wire_grid.cell(x, y).value(color).is_terminal() {
                            if let Some(output) = wire_grid.wire_bundle_output_mut(bundle_id) {
                                let stored_signals =
                                    &mut output.bundled_cells[*sub_bundle as usize];
                                let signal = &mut stored_signals[color as usize];

                                // Power is also consumed.
                                let logic = signal.logic;
                                let power = signal.power.take();

                                let cell = wire_grid.cell_mut(x, y).value_mut(color);

                                if let Some(power) = power {
                                    cell.set_power(power);
                                } else if let Some(logic) = logic {
                                    cell.set_logic(logic);
                                }
                            }
                        }
                    }
                }
            }
            ObjectType::DockingConnectorTop {
                state,
                progress,
                connected,
                previous_connected,
            } => {
                let cell_x = object.position.0 as usize + 20;
                let cell_y = object.position.1 as usize + 6;

                if !*previous_connected && *connected {
                    *state = DoorState::Opening;
                    wire_grid.cell_mut(cell_x, cell_y).send_logic(100);
                }

                if *previous_connected && !*connected {
                    *state = DoorState::Closing;
                    wire_grid.cell_mut(cell_x, cell_y).send_logic(-100);
                }

                *previous_connected = *connected;

                match state {
                    DoorState::Opening => *progress = (*progress + 1).min(15),
                    DoorState::Closing => *progress = progress.saturating_sub(1),
                };

                for x in 4..=17 {
                    for y in 2..=6 {
                        let cell = water_grid.cell_mut(
                            object.position.0 as usize + x,
                            object.position.1 as usize + y,
                        );
                        let frame = (*progress as u16 * 9 / 15).clamp(0, 8);

                        let top_y = match frame {
                            0..=2 => 5,
                            3..=5 => 4,
                            6..=7 => 3,
                            _ => 2,
                        };

                        // Extend the inside of the submarine for however far
                        // the docking connector is extended.
                        let above = y < top_y;
                        let top_wall = y == top_y;
                        let side_wall = x == 4 || x == 17;

                        // Stop sea water from coming through if connected to
                        // another submarine's grid.
                        let invisible_wall = top_wall && y == 2;
                        let open_wall = invisible_wall && !*connected;

                        if above || open_wall {
                            if !cell.is_sea() {
                                cell.make_sea();
                            }
                        } else if invisible_wall {
                            if !cell.is_wall() {
                                cell.make_invisible_wall();
                            }
                        } else if top_wall || side_wall {
                            if !cell.is_wall() {
                                cell.make_wall();
                            }
                        } else {
                            if !cell.is_inside() {
                                cell.make_inside();
                            }
                        }
                    }
                }

                *walls_updated = true;
            }
            ObjectType::DockingConnectorBottom {
                state,
                progress,
                connected,
                previous_connected,
            } => {
                let cell_x = object.position.0 as usize + 20;
                let cell_y = object.position.1 as usize + 4;

                if !*previous_connected && *connected {
                    *state = DoorState::Opening;
                    wire_grid.cell_mut(cell_x, cell_y).send_logic(100);
                }

                if *previous_connected && !*connected {
                    *state = DoorState::Closing;
                    wire_grid.cell_mut(cell_x, cell_y).send_logic(-100);
                }

                *previous_connected = *connected;

                match state {
                    DoorState::Opening => *progress = (*progress + 1).min(15),
                    DoorState::Closing => *progress = progress.saturating_sub(1),
                };

                for x in 4..=17 {
                    for y in 3..=7 {
                        let cell = water_grid.cell_mut(
                            object.position.0 as usize + x,
                            object.position.1 as usize + y,
                        );
                        let frame = (*progress as u16 * 9 / 15).clamp(0, 8);

                        let bottom_y = match frame {
                            0..=2 => 4,
                            3..=5 => 5,
                            6..=7 => 6,
                            _ => 7,
                        };

                        // Extend the inside of the submarine for however far
                        // the docking connector is extended.
                        let below = y > bottom_y;
                        let bottom_wall = y == bottom_y;
                        let side_wall = x == 4 || x == 17;

                        // Stop sea water from coming through if connected to
                        // another submarine's grid.
                        let invisible_wall = bottom_wall && y == 7;
                        let open_wall = invisible_wall && !*connected;

                        if below || open_wall {
                            if !cell.is_sea() {
                                cell.make_sea();
                            }
                        } else if invisible_wall {
                            if !cell.is_wall() {
                                cell.make_invisible_wall();
                            }
                        } else if bottom_wall || side_wall {
                            if !cell.is_wall() {
                                cell.make_wall();
                            }
                        } else {
                            if !cell.is_inside() {
                                cell.make_inside();
                            }
                        }
                    }
                }

                *walls_updated = true;
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
        ObjectType::JunctionBox { enabled, .. } => *enabled = !*enabled,
        ObjectType::NavController { active, .. } => *active = !*active,
        ObjectType::Sonar { active, .. } => *active = !*active,
        ObjectType::Engine { target_speed, .. } => cycle_i8(target_speed),
        ObjectType::Battery { .. } => (),
        ObjectType::BundleInput { sub_bundle } | ObjectType::BundleOutput { sub_bundle } => {
            *sub_bundle = (*sub_bundle + 1) % 8;
        }
        ObjectType::DockingConnectorTop { state, .. } => {
            *state = match state {
                DoorState::Opening => DoorState::Closing,
                DoorState::Closing => DoorState::Opening,
            }
        }
        ObjectType::DockingConnectorBottom { state, .. } => {
            *state = match state {
                DoorState::Opening => DoorState::Closing,
                DoorState::Closing => DoorState::Opening,
            }
        }
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
    let current_frame_column = 0;
    let powered = &object.powered;

    let current_frame = match &object.object_type {
        ObjectType::Door {
            progress, state, ..
        } => {
            let powered_offset = match (*powered, state) {
                (false, _) => 0,
                (true, DoorState::Closing) => 8,
                (true, DoorState::Opening) => 16,
            };

            (*progress as u16 * 8 / 15).clamp(0, 7) + powered_offset
        }
        ObjectType::VerticalDoor { progress, .. } => (*progress as u16 * 9 / 15).clamp(0, 8),
        ObjectType::Reactor { active } => {
            if *active {
                0
            } else {
                1
            }
        }
        ObjectType::Lamp => {
            if *powered {
                1
            } else {
                0
            }
        }
        ObjectType::Gauge { value } => match *value {
            -128..=-96 => 0,
            -95..=-32 => 1,
            -31..=31 => 2,
            32..=95 => 3,
            96..=127 => 4,
        },
        ObjectType::SmallPump { progress, .. } => {
            (*progress as u8 / (u8::MAX / 4)).clamp(0, 3) as u16
        }
        ObjectType::LargePump { progress, .. } => {
            (*progress as u8 / (u8::MAX / 4)).clamp(0, 3) as u16
        }
        ObjectType::JunctionBox { progress, .. } => {
            let powered_offset = if *powered { 0 } else { 5 };
            (*progress * 5 / 16).min(4) as u16 + powered_offset
        }
        ObjectType::NavController {
            active, progress, ..
        } => {
            if *active && *powered {
                (*progress as u16 / 8) % 5 + 1
            } else {
                0
            }
        }
        ObjectType::Sonar { active, .. } => {
            if *powered && *active {
                0
            } else {
                1
            }
        }
        ObjectType::Engine { progress, .. } => {
            let frames = 24;
            (*progress as u8 / (u8::MAX / frames)).clamp(0, frames - 1) as u16
        }
        ObjectType::Battery { charge } => {
            // Treat anything that isn't exactly 0 as having at least one blip
            // of power.
            if *charge == 0 {
                7
            } else {
                7 - (*charge * 8 / 5400).clamp(1, 7)
            }
        }
        ObjectType::BundleInput { sub_bundle } => *sub_bundle as u16,
        ObjectType::BundleOutput { sub_bundle } => *sub_bundle as u16,
        ObjectType::DockingConnectorTop { progress, .. } => {
            (*progress as u16 * 9 / 15).clamp(0, 8) + if *powered { 8 } else { 0 }
        }
        ObjectType::DockingConnectorBottom { progress, .. } => {
            (*progress as u16 * 9 / 15).clamp(0, 8) + if *powered { 8 } else { 0 }
        }
    };

    (current_frame, current_frame_column)
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
            ObjectType::Lamp { .. } => ObjectTypeTemplate::Lamp,
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
            ObjectType::JunctionBox { enabled, progress } => {
                ObjectTypeTemplate::JunctionBox { enabled, progress }
            }
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
            ObjectType::BundleInput { sub_bundle } => {
                ObjectTypeTemplate::BundleInput { sub_bundle }
            }
            ObjectType::BundleOutput { sub_bundle } => {
                ObjectTypeTemplate::BundleOutput { sub_bundle }
            }
            ObjectType::DockingConnectorTop {
                state,
                progress,
                connected,
                previous_connected,
            } => ObjectTypeTemplate::DockingConnectorTop {
                state,
                progress,
                connected,
                previous_connected,
            },
            ObjectType::DockingConnectorBottom {
                state,
                progress,
                connected,
                previous_connected,
            } => ObjectTypeTemplate::DockingConnectorBottom {
                state,
                progress,
                connected,
                previous_connected,
            },
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
            ObjectTypeTemplate::JunctionBox { enabled, progress } => {
                ObjectType::JunctionBox { enabled, progress }
            }
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
            ObjectTypeTemplate::BundleInput { sub_bundle } => {
                ObjectType::BundleInput { sub_bundle }
            }
            ObjectTypeTemplate::BundleOutput { sub_bundle } => {
                ObjectType::BundleOutput { sub_bundle }
            }
            ObjectTypeTemplate::DockingConnectorTop {
                state,
                progress,
                connected,
                previous_connected,
            } => ObjectType::DockingConnectorTop {
                state,
                progress,
                connected,
                previous_connected,
            },
            ObjectTypeTemplate::DockingConnectorBottom {
                state,
                progress,
                connected,
                previous_connected,
            } => ObjectType::DockingConnectorBottom {
                state,
                progress,
                connected,
                previous_connected,
            },
        };

        Object {
            object_type,
            position: self.position,
            powered: false,
        }
    }
}
