use std::{collections::BTreeMap, convert::TryInto};

use serde::{Deserialize, Serialize};

/// Logic and power wire grid.

// Still need to implement voltage/demand-based current and supply.

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct WireGrid {
    cells: Vec<WireCell>,
    width: usize,
    height: usize,
    connected_wires: [Vec<(usize, usize)>; WIRE_COLORS],
    bundle_inputs: Vec<WireBundle>,
    bundle_outputs: Vec<WireBundle>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct WireBundle {
    pub bundled_cells: [[StoredSignal; WIRE_COLORS]; 8],
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct StoredSignal {
    pub logic: Option<i8>,
    pub power: Option<u8>,
}

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct WireCell {
    value: [WireValue; WIRE_COLORS],
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) enum WireValue {
    NotConnected,
    NoSignal {
        terminal: bool,
    },
    Power {
        value: u8,
        terminal: bool,
        signal: u16,
    },
    Logic {
        value: i8,
        terminal: bool,
        signal: u16,
    },
    Bundle {
        bundle_id: u8,
    },
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum WireColor {
    Bundle = 0,
    Purple = 1,
    Brown = 2,
    Blue = 3,
    Green = 4,
}

pub(crate) type WirePoints = (WireColor, Vec<(usize, usize)>);

const NEIGHBOUR_OFFSETS: &[(i32, i32)] = &[(1, 0), (0, 1), (-1, 0), (0, -1)];

pub(crate) const WIRE_COLORS: usize = 5;

pub(crate) const THIN_COLORS: [WireColor; 4] = [
    WireColor::Purple,
    WireColor::Brown,
    WireColor::Blue,
    WireColor::Green,
];

impl Default for WireValue {
    fn default() -> Self {
        WireValue::NotConnected
    }
}

impl WireGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::new();
        cells.resize(width * height, WireCell::default());

        WireGrid {
            cells,
            width,
            height,
            connected_wires: Default::default(),
            bundle_inputs: Vec::new(),
            bundle_outputs: Vec::new(),
        }
    }

    pub fn clone_from(other_grid: &WireGrid) -> Self {
        let mut cells = Vec::new();
        cells.resize(other_grid.cells.len(), WireCell::default());

        for (color, wires) in other_grid.connected_wires.iter().enumerate() {
            for &(x, y) in wires {
                let old_cell: &WireCell = &other_grid.cells[y * other_grid.width + x];
                let new_cell: &mut WireCell = &mut cells[y * other_grid.width + x];
                new_cell.value[color] = old_cell.value[color];
            }
        }

        WireGrid {
            cells,
            width: other_grid.width,
            height: other_grid.height,
            connected_wires: other_grid.connected_wires.clone(),
            bundle_inputs: other_grid.bundle_inputs.clone(),
            bundle_outputs: other_grid.bundle_outputs.clone(),
        }
    }

    pub fn from_wire_points(width: usize, height: usize, wire_points: &[WirePoints]) -> Self {
        let mut wire_grid = WireGrid::new(width, height);

        for (color, wire_points) in wire_points {
            for pair in wire_points.windows(2) {
                let [(x1, y1), (x2, y2)] = match pair {
                    [p1, p2] => [p1, p2],
                    _ => unreachable!(),
                };

                let (x1, x2) = (x1.min(x2), x1.max(x2));
                let (y1, y2) = (y1.min(y2), y1.max(y2));

                for y in *y1..=*y2 {
                    for x in *x1..=*x2 {
                        wire_grid.make_wire(x, y, *color);
                    }
                }
            }
        }

        wire_grid
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn cell(&self, x: usize, y: usize) -> &WireCell {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        &self.cells[y * self.width + x]
    }

    pub fn cell_mut(&mut self, x: usize, y: usize) -> &mut WireCell {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        &mut self.cells[y * self.width + x]
    }

    pub fn make_wire(&mut self, x: usize, y: usize, color: WireColor) {
        self.cell_mut(x, y).value[color as usize] = if color == WireColor::Bundle {
            if let Some(bundle_id) = self.connect_bundle(x, y) {
                WireValue::Bundle { bundle_id }
            } else {
                return;
            }
        } else {
            WireValue::NoSignal { terminal: false }
        };
        if (1..self.width - 2).contains(&x) && (1..self.height - 1).contains(&y) {
            self.connected_wires[color as usize].push((x, y));
        }
    }

    pub fn clear_wire(&mut self, x: usize, y: usize, color: WireColor) {
        if color == WireColor::Bundle {
            // FIXME: Need to split bundles; which needs logic to detect a loop.
            return;
        }

        self.cell_mut(x, y).value[color as usize] = WireValue::NotConnected;
        if (1..self.width - 2).contains(&x) && (1..self.height - 1).contains(&y) {
            self.connected_wires[color as usize].retain(|wire| *wire != (x, y));
        }
    }

    fn connect_bundle(&mut self, x: usize, y: usize) -> Option<u8> {
        let mut neighbouring_sets = Vec::new();

        for neighbour in self.neighbours(x, y) {
            if let Some(bundle_id) = neighbour.bundle_id() {
                if !neighbouring_sets.contains(&bundle_id) {
                    neighbouring_sets.push(bundle_id);
                }
            }
        }

        let new_bundle_id = if let [one_neighbour] = neighbouring_sets[..] {
            one_neighbour
        } else {
            // No neighbours, or too many neighbours; make a new bundle to have
            // all of them.
            // Convert from usize to u8; if there's an overflow, don't create
            // any new bundles.
            if let Ok(bundle_id) = self.bundle_inputs.len().try_into() {
                assert_eq!(self.bundle_inputs.len(), self.bundle_outputs.len());
                self.bundle_inputs.push(WireBundle::default());
                self.bundle_outputs.push(WireBundle::default());
                bundle_id
            } else {
                return None;
            }
        };

        for neighbour_bundle_id in neighbouring_sets {
            for (color, wires) in &mut self.connected_wires.iter().enumerate() {
                if color != WireColor::Bundle as usize {
                    continue;
                }

                for &(x, y) in wires {
                    let cell = &mut self.cells[y * self.width + x];
                    let value = &mut cell.value[WireColor::Bundle as usize];

                    if let WireValue::Bundle { bundle_id } = value {
                        if *bundle_id == neighbour_bundle_id {
                            *bundle_id = new_bundle_id;
                        }
                    }
                }
            }
        }

        Some(new_bundle_id)
    }

    /// Returns whether a cell has the following neighbours: [down, right, up, left]
    pub fn has_neighbours(&self, wire_color: WireColor, x: usize, y: usize) -> [bool; 4] {
        let mut has_neighbours = [false; 4];

        for (index, (y_offset, x_offset)) in NEIGHBOUR_OFFSETS.iter().enumerate() {
            let cell = self.cell(
                (x as i32 + x_offset) as usize,
                (y as i32 + y_offset) as usize,
            );
            if cell.value[wire_color as usize].connected() {
                has_neighbours[index] = true;
            }
        }

        has_neighbours
    }

    fn neighbours(&self, x: usize, y: usize) -> impl Iterator<Item = &WireCell> {
        NEIGHBOUR_OFFSETS.iter().map(move |(y_offset, x_offset)| {
            self.cell(
                (x as i32 + x_offset) as usize,
                (y as i32 + y_offset) as usize,
            )
        })
    }

    pub fn update(&mut self, signals_updated: &mut bool) {
        let old_grid = WireGrid::clone_from(self);

        for (wire_color, wires) in self.connected_wires.iter().enumerate() {
            if wire_color == WireColor::Bundle as usize {
                // Wire bundles have instantaneous transmission and are updated
                // only when they're built/destroyed.
                continue;
            }

            for &(x, y) in wires {
                let cell = old_grid.cell(x, y);
                let old_value = &cell.value[wire_color];

                if !old_value.connected() {
                    continue;
                }

                let mut new_value = old_value.clone().decay(2);
                let mut connected_wires = 0;

                for neighbour in old_grid.neighbours(x, y) {
                    let neighbour_wire_value = &neighbour.value[wire_color];
                    if neighbour_wire_value.connected() {
                        connected_wires += 1;

                        if neighbour_wire_value.signal() > new_value.signal() + 3 {
                            new_value = neighbour_wire_value.decay(1);
                        }
                    }
                }

                if connected_wires > 2 {
                    new_value = WireValue::NotConnected;
                }

                new_value.set_terminal(connected_wires == 1);

                if self.cell(x, y).value[wire_color].signal() != new_value.signal() {
                    *signals_updated = true;
                }

                let cell_mut = &mut self.cells[y * self.width + x];
                cell_mut.value[wire_color] = new_value;
            }
        }
    }

    pub(crate) fn update_bundles(&mut self) {
        assert_eq!(self.bundle_inputs.len(), self.bundle_outputs.len());
        let zipped_bundles = self
            .bundle_inputs
            .iter_mut()
            .zip(self.bundle_outputs.iter_mut());

        for (input, output) in zipped_bundles {
            *output = input.clone();
            *input = Default::default();
        }
    }

    fn wire_sets(&self) -> Vec<(WireColor, Vec<(usize, usize)>)> {
        let mut wire_set_map = BTreeMap::new();
        let mut wire_sets: Vec<(WireColor, Vec<(usize, usize)>)> = Vec::new();

        let colors = [
            WireColor::Bundle,
            WireColor::Purple,
            WireColor::Brown,
            WireColor::Blue,
            WireColor::Green,
        ];

        for color in colors {
            for y in 0..self.height {
                for x in 0..self.width {
                    let wire_value = self.cell(x, y).value(color);

                    if wire_value.connected() {
                        let left_wire_set = if x > 0 {
                            wire_set_map.get(&(color, x - 1, y))
                        } else {
                            None
                        };

                        let top_wire_set = if y > 0 {
                            wire_set_map.get(&(color, x, y - 1))
                        } else {
                            None
                        };

                        let add_to_set = match (left_wire_set, top_wire_set) {
                            (None, None) => {
                                // Make a new set
                                wire_sets.push((color, Vec::new()));
                                wire_sets.len() - 1
                            }
                            (None, Some(&top_set)) => {
                                // Reuse the set from the cell above
                                let old_wires: &mut (WireColor, Vec<(usize, usize)>) =
                                    &mut wire_sets[top_set];
                                let last_wire =
                                    old_wires.1.last().expect("Sets have at least 1 wire");

                                if *last_wire != (x, y - 1) {
                                    // Make sure to connect to the correct end
                                    old_wires.1.reverse();
                                }

                                let last_wire =
                                    old_wires.1.last().expect("Sets have at least 1 wire");

                                if *last_wire != (x, y - 1) {
                                    // If this is still the incorrect end, then this is
                                    // a fork; make a new set.
                                    wire_sets.push((color, Vec::new()));
                                    wire_sets.len() - 1
                                } else {
                                    top_set
                                }
                            }
                            (Some(&left_set), None) => {
                                // Reuse the set from the cell to the left
                                let old_wires: &mut (WireColor, Vec<(usize, usize)>) =
                                    &mut wire_sets[left_set];
                                let last_wire =
                                    old_wires.1.last().expect("Sets have at least 1 wire");

                                if *last_wire != (x - 1, y) {
                                    // Make sure to connect to the correct end
                                    old_wires.1.reverse();
                                }
                                let last_wire =
                                    old_wires.1.last().expect("Sets have at least 1 wire");

                                if *last_wire != (x - 1, y) {
                                    // If this is still the incorrect end, then this is
                                    // a fork; make a new set.
                                    wire_sets.push((color, Vec::new()));
                                    wire_sets.len() - 1
                                } else {
                                    left_set
                                }
                            }
                            (Some(&left_set), Some(&top_set)) => {
                                // Merge the two sets
                                for &(old_x, old_y) in &wire_sets[top_set].1 {
                                    // Rewrite the top cells's set to match the top ones
                                    wire_set_map.insert((color, old_x, old_y), left_set);
                                }

                                let old_wires: &mut (WireColor, Vec<(usize, usize)>) =
                                    &mut wire_sets[top_set];
                                let last_wire =
                                    old_wires.1.last().expect("Sets have at least 1 wire");

                                if *last_wire != (x, y - 1) {
                                    // Make sure to connect to the correct end
                                    old_wires.1.reverse();
                                }

                                let last_wire =
                                    old_wires.1.last().expect("Sets have at least 1 wire");
                                if *last_wire != (x, y - 1) {
                                    // If this is still the incorrect end, then this is
                                    // a fork; make a new set.
                                    wire_sets.push((color, Vec::new()));
                                    wire_sets.len() - 1
                                } else {
                                    // Add the current cell, which will be sandwhiched between the older top
                                    // and newer left sets.
                                    wire_set_map.insert((color, x, y), left_set);
                                    wire_sets[left_set].1.push((x, y));

                                    // Add the top cells to the left set in reversed order
                                    // This is to keep neighbours contiguous in the list
                                    let old_wires: Vec<(usize, usize)> =
                                        wire_sets[top_set].1.iter().copied().rev().collect();
                                    wire_sets[left_set].1.extend(old_wires);

                                    wire_sets[top_set].1.clear();

                                    // Already added in the middle, don't return anything
                                    continue;
                                }
                            }
                        };

                        wire_set_map.insert((color, x, y), add_to_set);
                        wire_sets[add_to_set].1.push((x, y));
                    }
                }
            }
        }

        wire_sets.retain(|wire_set| !wire_set.1.is_empty());

        wire_sets
    }

    pub fn wire_points(&self) -> Vec<WirePoints> {
        let wire_sets = self.wire_sets();
        let mut wire_points = Vec::new();

        for (color, wire_set) in wire_sets {
            let points = wire_set_into_points(wire_set);
            wire_points.push((color, points));
        }

        wire_points
    }

    pub fn wire_bundle_input_mut(&mut self, bundle_id: u8) -> Option<&mut WireBundle> {
        let bundle_id: usize = bundle_id.into();
        self.bundle_inputs.get_mut(bundle_id)
    }

    pub fn wire_bundle_output_mut(&mut self, bundle_id: u8) -> Option<&mut WireBundle> {
        let bundle_id: usize = bundle_id.into();
        self.bundle_outputs.get_mut(bundle_id)
    }
}

// Given a contiguous list of wires, keep only the inflection points
fn wire_set_into_points(wire_set: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut last_direction = (0, 0);

    if wire_set.len() < 2 {
        return wire_set;
    }

    let mut points = Vec::new();

    for (index, next_wire) in wire_set[1..].iter().enumerate() {
        let previous_wire = wire_set[index];

        let new_direction = (
            next_wire.0 as i32 - previous_wire.0 as i32,
            next_wire.1 as i32 - previous_wire.1 as i32,
        );

        if new_direction != last_direction {
            points.push(previous_wire);
            last_direction = new_direction;
        }
    }

    points.push(*wire_set.last().expect("Checked length above"));

    points
}

impl WireCell {
    pub fn value(&self, color: WireColor) -> &WireValue {
        &self.value[color as usize]
    }

    pub fn value_mut(&mut self, color: WireColor) -> &mut WireValue {
        &mut self.value[color as usize]
    }

    pub fn receive_logic(&self) -> Option<i8> {
        for wire_color in 0..WIRE_COLORS {
            let value = self.value[wire_color];

            if value.is_terminal() {
                match value {
                    WireValue::NotConnected => (),
                    WireValue::NoSignal { .. } => (),
                    WireValue::Power { .. } => (),
                    WireValue::Logic { value, .. } => return Some(value),
                    WireValue::Bundle { .. } => (),
                };
            }
        }
        None
    }

    pub fn receive_power(&self) -> Option<u8> {
        for wire_color in 0..WIRE_COLORS {
            let value = self.value[wire_color];

            if value.is_terminal() {
                match value {
                    WireValue::NotConnected => (),
                    WireValue::NoSignal { .. } => (),
                    WireValue::Power { value, .. } => return Some(value),
                    WireValue::Logic { .. } => (),
                    WireValue::Bundle { .. } => (),
                }
            };
        }
        None
    }

    pub fn minimum_power(&self, minimum: u8) -> bool {
        match self.receive_power() {
            Some(value) => value >= minimum,
            None => false,
        }
    }

    pub fn send_logic(&mut self, logic_value: i8) {
        for wire_color in 0..WIRE_COLORS {
            let wire_value = &mut self.value[wire_color];

            if wire_value.connected() && wire_value.is_terminal() {
                *wire_value = WireValue::Logic {
                    value: logic_value,
                    signal: 256,
                    terminal: true,
                };
            }
        }
    }

    pub fn send_power(&mut self, power_value: u8) {
        for wire_color in 0..WIRE_COLORS {
            let wire_value = &mut self.value[wire_color];

            if wire_value.connected() && wire_value.is_terminal() {
                *wire_value = WireValue::Power {
                    value: power_value,
                    signal: 256,
                    terminal: true,
                };
                // Send to at most one wire.
                break;
            }
        }
    }

    pub fn bundle_id(&self) -> Option<u8> {
        if let WireValue::Bundle { bundle_id } = self.value(WireColor::Bundle) {
            Some(*bundle_id)
        } else {
            None
        }
    }
}

impl WireValue {
    pub fn signal(&self) -> u16 {
        match self {
            WireValue::NotConnected => 0,
            WireValue::NoSignal { .. } => 0,
            WireValue::Power { signal, .. } => *signal,
            WireValue::Logic { signal, .. } => *signal,
            WireValue::Bundle { .. } => 0,
        }
    }

    pub fn is_terminal(&self) -> bool {
        match self {
            WireValue::NotConnected => false,
            WireValue::NoSignal { terminal } => *terminal,
            WireValue::Power { terminal, .. } => *terminal,
            WireValue::Logic { terminal, .. } => *terminal,
            WireValue::Bundle { .. } => false,
        }
    }

    pub fn set_terminal(&mut self, is_terminal: bool) {
        match self {
            WireValue::NotConnected => (),
            WireValue::NoSignal { terminal } => *terminal = is_terminal,
            WireValue::Power { terminal, .. } => *terminal = is_terminal,
            WireValue::Logic { terminal, .. } => *terminal = is_terminal,
            WireValue::Bundle { .. } => (),
        }
    }

    pub fn set_logic(&mut self, value: i8) {
        if !self.connected() || !self.is_terminal() {
            return;
        }

        *self = WireValue::Logic {
            value,
            terminal: true,
            signal: 256,
        }
    }

    pub fn set_power(&mut self, value: u8) {
        if !self.connected() || !self.is_terminal() {
            return;
        }

        *self = WireValue::Power {
            value,
            terminal: true,
            signal: 256,
        }
    }

    pub fn get_logic(&self) -> Option<i8> {
        match self {
            WireValue::Logic {
                value,
                terminal: true,
                ..
            } => Some(*value),
            _ => None,
        }
    }

    pub fn get_power(&self) -> Option<u8> {
        match self {
            WireValue::Power {
                value,
                terminal: true,
                ..
            } => Some(*value),
            _ => None,
        }
    }

    fn decay(&self, amount: u16) -> WireValue {
        let new_signal = match self {
            WireValue::NotConnected => WireValue::NotConnected,
            WireValue::NoSignal { terminal } => WireValue::NoSignal {
                terminal: *terminal,
            },
            WireValue::Power {
                value,
                signal,
                terminal,
            } => WireValue::Power {
                value: *value,
                signal: signal.saturating_sub(amount),
                terminal: *terminal,
            },
            WireValue::Logic {
                value,
                signal,
                terminal,
            } => WireValue::Logic {
                value: *value,
                signal: signal.saturating_sub(amount),
                terminal: *terminal,
            },
            WireValue::Bundle { bundle_id } => WireValue::Bundle {
                bundle_id: *bundle_id,
            },
        };

        match new_signal {
            WireValue::Power {
                signal: 0,
                terminal,
                ..
            } => WireValue::NoSignal { terminal },
            WireValue::Logic {
                signal: 0,
                terminal,
                ..
            } => WireValue::NoSignal { terminal },
            value => value,
        }
    }

    pub fn connected(&self) -> bool {
        !matches!(self, &WireValue::NotConnected)
    }
}
