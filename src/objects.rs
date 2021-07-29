use crate::water::WaterGrid;

pub(crate) struct Object {
    pub object_type: ObjectType,
    pub position_x: u32,
    pub position_y: u32,
    pub current_frame: u16,
    pub frames: u16,
}

pub(crate) enum ObjectType {
    Door { state: DoorState, progress: u8 },
    VerticalDoor { state: DoorState, progress: u8 },
    // SmallPump,
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
        }
    }
}

// What an object does on every physics update tick.
pub(crate) fn update_objects(objects: &mut Vec<Object>, grid: &mut WaterGrid) {
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

                        let cell = grid.cell_mut(cell_x as usize, cell_y as usize);

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

                    let cell = grid.cell_mut(cell_x as usize, cell_y as usize);

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
    }
}
