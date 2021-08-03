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
        object_type: ObjectType::Reactor { active: false },
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

    objects
}

pub(crate) fn load_wires(grid: &mut WireGrid) {
    let wires = &[
        // Reactor to first junction box
        (141..=141, 71..=81, WireColor::Blue),
        (141..=183, 71..=71, WireColor::Blue),
        (183..=183, 71..=73, WireColor::Blue),
        // First junction box to lamp
        (186..=187, 74..=74, WireColor::Green),
        (187..=187, 71..=74, WireColor::Green),
        (163..=187, 71..=71, WireColor::Green),
        (163..=163, 71..=74, WireColor::Green),
        // First junction box to left large pump
        (186..=187, 75..=75, WireColor::Brown),
        (187..=187, 71..=75, WireColor::Brown),
        (78..=187, 71..=71, WireColor::Brown),
        (78..=78, 71..=80, WireColor::Brown),
        // First junction box to right large pump
        (186..=187, 76..=76, WireColor::Orange),
        (187..=187, 71..=76, WireColor::Orange),
        (187..=292, 71..=71, WireColor::Orange),
        (292..=292, 71..=80, WireColor::Orange),
        // Main gauge to second junction box
        (119..=119, 58..=60, WireColor::Green),
        (119..=124, 60..=60, WireColor::Green),
        (124..=124, 47..=60, WireColor::Green),
        (124..=224, 47..=47, WireColor::Green),
        (224..=224, 47..=71, WireColor::Green),
        (190..=224, 71..=71, WireColor::Green),
        (190..=190, 71..=72, WireColor::Green),
        // Second junction box to right pump gauge
        (193..=194, 75..=75, WireColor::Blue),
        (194..=194, 71..=75, WireColor::Blue),
        (194..=279, 71..=71, WireColor::Blue),
        (279..=279, 71..=73, WireColor::Blue),
        // Right pump gauge to pump
        (279..=279, 76..=78, WireColor::Green),
        (279..=282, 78..=78, WireColor::Green),
        (282..=282, 71..=78, WireColor::Green),
        (282..=295, 71..=71, WireColor::Green),
        (295..=295, 71..=80, WireColor::Green),
    ];

    for (x_range, y_range, color) in wires {
        for y in y_range.clone() {
            for x in x_range.clone() {
                let cell = grid.cell_mut(x, y);
                cell.make_wire(*color);
            }
        }
    }
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

    grid
}
