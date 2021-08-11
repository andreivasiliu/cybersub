use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Logic and power wire grid.

// Still need to implement cable bundles and voltage/demand-based current and supply.

#[derive(Clone)]
pub(crate) struct WireGrid {
    cells: Vec<WireCell>,
    width: usize,
    height: usize,
    connected_wires: [Vec<(usize, usize)>; COLORS],
}

#[derive(Default, Clone, Copy)]
pub(crate) struct WireCell {
    value: [WireValue; COLORS],
}

#[derive(Clone, Copy)]
pub(crate) enum WireValue {
    NotConnected,
    NoSignal,
    Power { value: u8, signal: u16 },
    Logic { value: i8, signal: u16 },
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum WireColor {
    Purple = 0,
    Brown = 1,
    Blue = 2,
    Green = 3,
}

const NEIGHBOUR_OFFSETS: &[(i32, i32)] = &[(1, 0), (0, 1), (-1, 0), (0, -1)];

const COLORS: usize = 4;

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
        }
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
        self.cell_mut(x, y).value[color as usize] = WireValue::NoSignal;
        if (1..self.width - 2).contains(&x) && (1..self.height - 1).contains(&y) {
            self.connected_wires[color as usize].push((x, y));
        }
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

                if self.cell(x, y).value[wire_color].signal() != new_value.signal() {
                    *signals_updated = true;
                }

                let cell_mut = &mut self.cells[y * self.width + x];
                cell_mut.value[wire_color] = new_value;
            }
        }
    }

    pub fn wire_sets(&self) -> Vec<(WireColor, Vec<(usize, usize)>)> {
        let mut wire_set_map = BTreeMap::new();
        let mut wire_sets: Vec<(WireColor, Vec<(usize, usize)>)> = Vec::new();

        let colors = [
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
                                let new_set = wire_sets.len() - 1;
                                new_set
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

                                top_set
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
                                left_set
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

                                // Already addeed in the middle, don't return anything
                                continue;
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

    pub(crate) fn wire_points(&self) -> Vec<(WireColor, Vec<(usize, usize)>)> {
        let wire_sets = self.wire_sets();
        let mut wire_points = Vec::new();

        for (color, wire_set) in wire_sets {
            let points = wire_set_into_points(wire_set);
            wire_points.push((color, points));
        }

        wire_points
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

    pub fn receive_logic(&self) -> Option<i8> {
        for wire_color in 0..COLORS {
            match self.value[wire_color] {
                WireValue::NotConnected => (),
                WireValue::NoSignal => (),
                WireValue::Power { .. } => (),
                WireValue::Logic { value, .. } => return Some(value),
            };
        }
        None
    }

    pub fn receive_power(&self) -> Option<u8> {
        for wire_color in 0..COLORS {
            match self.value[wire_color] {
                WireValue::NotConnected => (),
                WireValue::NoSignal => (),
                WireValue::Power { value, .. } => return Some(value),
                WireValue::Logic { .. } => (),
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
        for wire_color in 0..COLORS {
            let wire_value = &mut self.value[wire_color];

            if wire_value.connected() {
                *wire_value = WireValue::Logic {
                    value: logic_value,
                    signal: 256,
                };
            }
        }
    }

    pub fn send_power(&mut self, power_value: u8) {
        for wire_color in 0..COLORS {
            let wire_value = &mut self.value[wire_color];

            if wire_value.connected() {
                *wire_value = WireValue::Power {
                    value: power_value,
                    signal: 256,
                };
                // Send to at most one wire.
                break;
            }
        }
    }
}

impl WireValue {
    pub fn signal(&self) -> u16 {
        match self {
            WireValue::NotConnected => 0,
            WireValue::NoSignal => 0,
            WireValue::Power { signal, .. } => *signal,
            WireValue::Logic { signal, .. } => *signal,
        }
    }

    fn decay(&self, amount: u16) -> WireValue {
        let new_signal = match self {
            WireValue::NotConnected => WireValue::NotConnected,
            WireValue::NoSignal => WireValue::NoSignal,
            WireValue::Power { value, signal } => WireValue::Power {
                value: *value,
                signal: signal.saturating_sub(amount),
            },
            WireValue::Logic { value, signal } => WireValue::Logic {
                value: *value,
                signal: signal.saturating_sub(amount),
            },
        };

        match new_signal {
            WireValue::NotConnected => WireValue::NotConnected,
            WireValue::NoSignal => WireValue::NoSignal,
            WireValue::Power { signal: 0, .. } => WireValue::NoSignal,
            WireValue::Logic { signal: 0, .. } => WireValue::NoSignal,
            value => value,
        }
    }

    pub fn connected(&self) -> bool {
        !matches!(self, &WireValue::NotConnected)
    }
}
