use std::io::Read;

use flate2::read::GzDecoder;
use macroquad::prelude::{Image, ImageFormat, BLACK};
use png::{BitDepth, ColorType, Decoder, Encoder};

use crate::{
    objects::{DoorState, Object, ObjectType},
    rocks::{RockGrid, RockType},
    water::WaterGrid,
    wires::{WireColor, WireGrid},
};

pub(crate) fn save(grid: &WaterGrid) -> Result<(), String> {
    if cfg!(target_arch = "wasm32") {
        return Err("Saving not yet possible on browsers".to_string());
    }

    use flate2::{read::GzEncoder, Compression};

    let file =
        std::fs::File::create("grid.bin.gz").map_err(|err| format!("Could not save: {}", err))?;
    let encoder = GzEncoder::new(file, Compression::best());
    let writer = std::io::BufWriter::new(encoder);

    bincode::serialize_into(writer, grid)
        .map_err(|err| format!("Could not serialize grid: {}", err))?;

    Ok(())
}

pub(crate) fn load() -> Result<WaterGrid, String> {
    if cfg!(target_arch = "wasm32") {
        return Err("Loading not yet possible on browsers".to_string());
    }

    let file =
        std::fs::File::open("grid.bin.gz").map_err(|err| format!("Could not load: {}", err))?;
    let decoder = GzDecoder::new(file);
    let reader = std::io::BufReader::new(decoder);

    let grid = bincode::deserialize_from(reader)
        .map_err(|err| format!("Could not deserialize: {}", err))?;

    Ok(grid)
}

pub(crate) fn save_png(grid: &WaterGrid) -> Result<(), String> {
    if cfg!(target_arch = "wasm32") {
        return Err("Saving not yet possible on browsers".to_string());
    }

    let file =
        std::fs::File::create("grid.png").map_err(|err| format!("Could not save: {}", err))?;
    let writer = std::io::BufWriter::new(file);

    let (width, height) = grid.size();
    let mut png_encoder = Encoder::new(writer, width as u32, height as u32);
    png_encoder.set_color(ColorType::RGBA);
    png_encoder.set_depth(BitDepth::Eight);

    let mut png_writer = png_encoder
        .write_header()
        .map_err(|err| format!("Could not write PNG header: {}", err))?;
    let mut data = Vec::with_capacity(width * height * 4);

    for y in 0..height {
        for x in 0..width {
            let cell = grid.cell(x, y);

            let pixel = if cell.is_wall() {
                [255, 255, 255, 255]
            } else if cell.amount_overfilled() > 0.5 {
                [0, 0, 255, 255]
            } else {
                [0, 0, 0, 0]
            };

            data.extend_from_slice(&pixel);
        }
    }

    png_writer
        .write_image_data(&data)
        .map_err(|err| format!("Could not write PNG data: {}", err))?;

    Ok(())
}

fn load_png_from_decoder(png_decoder: Decoder<impl Read>) -> Result<WaterGrid, String> {
    let (png_info, mut png_reader) = png_decoder
        .read_info()
        .map_err(|err| format!("Could not read PNG header: {}", err))?;

    let (width, height) = (png_info.width as usize, png_info.height as usize);
    let mut grid = WaterGrid::new(width, height);

    if png_info.bit_depth != BitDepth::Eight {
        return Err("PNG must be RGBA with 8 bits per channel".to_owned());
    }

    for y in 0..height {
        let data = png_reader
            .next_row()
            .map_err(|err| format!("Error reading PNG row: {}", err))?
            .expect("Expected row count to equal PNG height");

        for x in 0..width {
            let cell = grid.cell_mut(x, y);

            match data[x * 4..x * 4 + 4] {
                [0, 0, 255, 255] => cell.make_sea(),
                [255, 255, 255, 255] => cell.make_wall(),
                [0, 0, 0, 0] => cell.make_inside(),
                [r, g, b, a] => {
                    return Err(format!(
                    "Unknown color code {}/{}/{}/{}; expected blue, white, or black-transparent",
                    r, g, b, a
                ))
                }
                _ => panic!("Expected row size to equal PNG width"),
            }
        }
    }

    Ok(grid)
}

pub(crate) fn load_png_from_bytes(bytes: &[u8]) -> Result<WaterGrid, String> {
    let reader = std::io::BufReader::new(bytes);
    let png_decoder = Decoder::new(reader);

    load_png_from_decoder(png_decoder)
}

pub(crate) fn load_png() -> Result<WaterGrid, String> {
    if cfg!(target_arch = "wasm32") {
        return Err("Loading not yet possible on browsers".to_string());
    }

    let file = std::fs::File::open("grid.png").map_err(|err| format!("Could not load: {}", err))?;
    let reader = std::io::BufReader::new(file);
    let png_decoder = Decoder::new(reader);

    load_png_from_decoder(png_decoder)
}

pub(crate) fn load_objects() -> Vec<Object> {
    let mut objects = Vec::new();

    let doors = &[(146, 13), (191, 39), (209, 64), (273, 64), (59, 64)];

    for door in doors {
        objects.push(Object {
            object_type: ObjectType::Door {
                state: DoorState::Closing,
                progress: 0,
            },
            position_x: door.0,
            position_y: door.1,
            current_frame: 0,
            frames: 8,
        });
    }

    let vertical_doors = &[
        (167, 23),
        (77, 48),
        (189, 48),
        (267, 48),
        (313, 48),
        (173, 76),
        (231, 76),
    ];

    for door in vertical_doors {
        objects.push(Object {
            object_type: ObjectType::VerticalDoor {
                state: DoorState::Closing,
                progress: 0,
            },
            position_x: door.0,
            position_y: door.1,
            current_frame: 0,
            frames: 9,
        });
    }

    objects.push(Object {
        object_type: ObjectType::Reactor { active: true },
        position_x: 112,
        position_y: 76,
        current_frame: 1,
        frames: 2,
    });

    objects.push(Object {
        object_type: ObjectType::Lamp,
        position_x: 160,
        position_y: 73,
        current_frame: 0,
        frames: 2,
    });

    let gauges = &[(115, 52), (62, 71), (275, 71)];

    for gauge in gauges {
        objects.push(Object {
            object_type: ObjectType::Gauge { value: 0 },
            position_x: gauge.0,
            position_y: gauge.1,
            current_frame: 2,
            frames: 5,
        });
    }

    let pumps = &[(68, 76), (282, 76)];

    for pump in pumps {
        objects.push(Object {
            object_type: ObjectType::LargePump {
                target_speed: 0,
                speed: 0,
                progress: 0,
            },
            position_x: pump.0,
            position_y: pump.1,
            current_frame: 0,
            frames: 4,
        });
    }

    let junction_boxes = &[
        (180, 71),
        (187, 71),
        (194, 71),
        (201, 71),
        (208, 71),
        (215, 71),
        (187, 80),
        (194, 80),
    ];

    for junction_box in junction_boxes {
        objects.push(Object {
            object_type: ObjectType::JunctionBox,
            position_x: junction_box.0,
            position_y: junction_box.1,
            current_frame: 0,
            frames: 1,
        });
    }

    objects.push(Object {
        object_type: ObjectType::NavController {
            active: true,
            progress: 0,
        },
        position_x: 95,
        position_y: 50,
        current_frame: 0,
        frames: 6,
    });

    objects.push(Object {
        object_type: ObjectType::Sonar { active: true },
        position_x: 130,
        position_y: 48,
        current_frame: 0,
        frames: 2,
    });

    objects
}

pub(crate) fn load_wires(width: usize, height: usize) -> WireGrid {
    let mut grid = WireGrid::new(width, height);

    let wires = &[
        // Reactor to first junction box
        (
            WireColor::Blue,
            &[(141, 81), (141, 71), (183, 71), (183, 73)][..],
        ),
        // First junction box to lamp
        (
            WireColor::Green,
            &[(186, 74), (187, 74), (187, 71), (163, 71), (163, 74)],
        ),
        // First junction box to left large pump
        (
            WireColor::Brown,
            &[(186, 75), (187, 75), (187, 71), (78, 71), (78, 80)],
        ),
        // First junction box to right large pump
        (
            WireColor::Orange,
            &[(186, 76), (187, 76), (187, 71), (292, 71), (292, 80)],
        ),
        // First junctin box to second junction box
        (
            WireColor::Blue,
            &[(186, 77), (187, 77), (187, 71), (190, 71), (190, 72)],
        ),
        // Main gauge to right pump gauge
        (
            WireColor::Green,
            &[
                (119, 58),
                (119, 60),
                (124, 60),
                (124, 46),
                (224, 46),
                (224, 71),
                (279, 71),
                (279, 73),
            ],
        ),
        // Main gauge to left pump gauge
        (
            WireColor::Orange,
            &[
                (119, 58),
                (119, 60),
                (114, 60),
                (114, 46),
                (75, 46),
                (75, 71),
                (66, 71),
                (66, 73),
            ],
        ),
        // Left pump gauge to pump
        (
            WireColor::Green,
            &[(66, 77), (66, 79), (71, 79), (71, 71), (81, 71), (81, 79)],
        ),
        // Right pump gauge to pump
        (
            WireColor::Green,
            &[
                (279, 76),
                (279, 78),
                (282, 78),
                (282, 71),
                (295, 71),
                (295, 80),
            ],
        ),
        // Second junction box to nav controller
        (
            WireColor::Brown,
            &[
                (193, 74),
                (194, 74),
                (194, 71),
                (224, 71),
                (224, 46),
                (96, 46),
                (96, 54),
                (97, 54),
            ],
        ),
        // Nav controller to main gauge
        (
            WireColor::Blue,
            &[(103, 54), (115, 54), (115, 53), (119, 53), (119, 54)],
        ),
    ];

    for (color, wire_points) in wires {
        for pair in wire_points.windows(2) {
            let [(x1, y1), (x2, y2)] = match pair {
                [p1, p2] => [p1, p2],
                _ => unreachable!(),
            };

            let (x1, x2) = (x1.min(x2), x1.max(x2));
            let (y1, y2) = (y1.min(y2), y1.max(y2));

            for y in *y1..=*y2 {
                for x in *x1..=*x2 {
                    let cell = grid.cell_mut(x, y);
                    cell.make_wire(*color);
                }
            }
        }
    }

    grid
}

pub(crate) fn load_rocks_from_png(bytes: &[u8]) -> RockGrid {
    let image = Image::from_file_with_format(bytes, Some(ImageFormat::Png));
    load_rocks_from_image(image)
}

fn load_rocks_from_image(image: Image) -> RockGrid {
    let width = image.width() / 2;
    let height = image.height() / 2;

    let mut grid = RockGrid::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let cell = grid.cell_mut(x, y);

            let (x, y) = (x as u32, y as u32);
            let colors = [
                image.get_pixel(x * 2, y * 2) == BLACK,
                image.get_pixel(x * 2 + 1, y * 2) == BLACK,
                image.get_pixel(x * 2, y * 2 + 1) == BLACK,
                image.get_pixel(x * 2 + 1, y * 2 + 1) == BLACK,
            ];

            let rock_type = match colors {
                // Upper-left, upper-right, lower-left, lower-right
                [false, false, false, false] => RockType::Empty,
                [true, true, true, true] => RockType::WallFilled,
                [true, false, true, true] => RockType::WallLowerLeft,
                [false, false, true, false] => RockType::WallLowerLeft,
                [false, true, true, true] => RockType::WallLowerRight,
                [false, false, false, true] => RockType::WallLowerRight,
                [true, true, true, false] => RockType::WallUpperLeft,
                [true, false, false, false] => RockType::WallUpperLeft,
                [true, true, false, true] => RockType::WallUpperRight,
                [false, true, false, false] => RockType::WallUpperRight,
                _ => RockType::WallFilled,
            };

            cell.set_type(rock_type);
        }
    }

    grid.update_edges();

    grid
}
