/// Logic and power wire grid.

// Still need to implement cable bundles and voltage/demand-based current and supply.

#[derive(Clone)]
pub(crate) struct WireGrid {
    cells: Vec<WireCell>,
    width: usize,
    height: usize,
}

#[derive(Default, Clone)]
pub(crate) struct WireCell {
    value: [WireValue; COLORS],
}

#[derive(Clone)]
pub(crate) enum WireValue {
    NotConnected,
    NoSignal,
    Power { value: u8, signal: u16 },
    Logic { value: i8, signal: u16 },
}

#[derive(Clone, Copy)]
pub(crate) enum WireColor {
    Orange = 0,
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

    pub fn update(&mut self) {
        let old_grid = self.clone();

        for y in 1..old_grid.height - 1 {
            for x in 1..old_grid.width - 1 {
                let cell = old_grid.cell(x, y);

                for wire_color in 0..COLORS {
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

                    self.cell_mut(x, y).value[wire_color] = new_value;
                }
            }
        }
    }
}

impl WireCell {
    pub fn make_wire(&mut self, color: WireColor) {
        self.value[color as usize] = WireValue::NoSignal;
    }

    pub fn make_powered_wire(&mut self, color: WireColor) {
        self.value[color as usize] = WireValue::Power {
            value: 200,
            signal: 256,
        };
    }

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

    fn connected(&self) -> bool {
        !matches!(self, &WireValue::NotConnected)
    }
}
