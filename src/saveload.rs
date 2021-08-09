use std::io::Read;

use flate2::read::GzDecoder;
use macroquad::prelude::{Image, ImageFormat, Texture2D, BLACK};
use png::{BitDepth, ColorType, Decoder, Encoder};

use crate::{
    objects::Object,
    rocks::{RockGrid, RockType},
    water::WaterGrid,
    wires::{WireColor, WireGrid},
};

pub struct SubmarineFileData {
    pub water_grid: Vec<u8>,
    pub background: Vec<u8>,
    pub objects: Vec<u8>,
}

pub(crate) fn load_from_file_data(
    files: SubmarineFileData,
) -> Result<(WaterGrid, Texture2D, Vec<Object>), String> {
    let water_grid = load_png_from_bytes(&files.water_grid)?;
    let objects = load_objects_from_yaml(&files.objects)?;
    let background = Texture2D::from_file_with_format(&files.background, Some(ImageFormat::Png));

    Ok((water_grid, background, objects))
}

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

    // Edges don't need to be updated ever again after this.
    grid.update_edges();

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

pub(crate) fn load_wires(width: usize, height: usize) -> WireGrid {
    let mut grid = WireGrid::new(width, height);

    let wires = &[
        // Reactor to first junction box
        (
            WireColor::Green,
            &[(141, 81), (141, 71), (183, 71), (183, 73)][..],
        ),
        // First junction box to lamp
        (
            WireColor::Purple,
            &[(186, 74), (187, 74), (187, 71), (163, 71), (163, 74)],
        ),
        // First junction box to left large pump
        (
            WireColor::Brown,
            &[(186, 75), (187, 75), (187, 71), (78, 71), (78, 80)],
        ),
        // Second junction box to right large pump
        (
            WireColor::Purple,
            &[(193, 77), (194, 77), (194, 71), (292, 71), (292, 80)],
        ),
        // First junctin box to second junction box
        (
            WireColor::Green,
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
            WireColor::Purple,
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
        // Second junction box to sonar display
        (
            WireColor::Blue,
            &[
                (193, 76),
                (194, 76),
                (194, 71),
                (224, 71),
                (224, 46),
                (131, 46),
                (131, 63),
                (132, 63),
            ],
        ),
        // First junction box to engine
        (
            WireColor::Blue,
            &[
                (186, 76),
                (187, 76),
                (187, 71),
                (39, 71),
                (39, 67),
                (37, 67),
            ],
        ),
        // Nav controller to engine
        (
            WireColor::Green,
            &[
                (103, 56),
                (104, 56),
                (104, 46),
                (64, 46),
                (64, 47),
                (51, 47),
                (51, 48),
                (46, 48),
                (46, 49),
                (42, 49),
                (42, 50),
                (39, 50),
                (39, 69),
                (37, 69),
            ],
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
                    grid.make_wire(x, y, *color);
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

pub(crate) fn load_objects_from_yaml(object_bytes: &[u8]) -> Result<Vec<Object>, String> {
    serde_yaml::from_slice(object_bytes).map_err(|err| err.to_string())
}
