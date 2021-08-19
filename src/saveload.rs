use std::{
    io::{Read, Write},
    path::Path,
};

use flate2::read::GzDecoder;
use macroquad::prelude::{Image, ImageFormat, BLACK};
use png::{BitDepth, ColorType, Decoder, Encoder};

use crate::{
    game_state::objects::Object,
    game_state::rocks::{RockGrid, RockType},
    game_state::state::SubmarineState,
    game_state::water::{WallMaterial, WaterGrid},
    game_state::wires::{WireColor, WireGrid},
    resources::MutableSubResources,
};

pub struct SubmarineFileData {
    pub water_grid: Vec<u8>,
    pub background: Vec<u8>,
    pub objects: Vec<u8>,
    pub wires: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct SubmarineData {
    pub water_grid: WaterGrid,
    pub background: Image,
    pub objects: Vec<Object>,
    pub wire_grid: WireGrid,
}

pub(crate) fn load_from_file_data(file_data: SubmarineFileData) -> Result<SubmarineData, String> {
    let water_grid = load_water_from_png(&file_data.water_grid)?;
    let objects = load_objects_from_yaml(&file_data.objects)?;

    let background = Image::from_file_with_format(&file_data.background, Some(ImageFormat::Png));

    let (width, height) = water_grid.size();
    let wire_grid = load_wires_from_yaml(&file_data.wires, width, height)?;

    Ok(SubmarineData {
        water_grid,
        background,
        objects,
        wire_grid,
    })
}

pub(crate) fn save_to_file_data(
    submarine: &SubmarineState,
    resources: &MutableSubResources,
) -> Result<SubmarineFileData, String> {
    let wires = save_wires_to_yaml(&submarine.wire_grid)?;
    let water_grid = save_water_to_png(&submarine.water_grid)?;
    let objects = save_objects_to_yaml(&submarine.objects)?;
    let background = image_to_png(&resources.sub_background_image)?;

    Ok(SubmarineFileData {
        water_grid,
        background,
        wires,
        objects,
    })
}

pub(crate) fn load_from_directory(path: &str) -> Result<SubmarineFileData, String> {
    let read_file = |file_name| {
        std::fs::read(format!("{}/{}", path, file_name))
            .map_err(|err| format!("Could not open file {} in {}: {}", file_name, path, err))
    };

    Ok(SubmarineFileData {
        water_grid: read_file("water_grid.png")?,
        background: read_file("background.png")?,
        objects: read_file("objects.yaml")?,
        wires: read_file("wires.yaml")?,
    })
}

pub(crate) fn save_to_directory(
    path: &str,
    file_data: SubmarineFileData,
    overwrite: bool,
) -> Result<(), String> {
    let file_names = &[
        ("wires.yaml", &file_data.wires),
        ("water_grid.png", &file_data.water_grid),
        ("objects.yaml", &file_data.objects),
        ("background.png", &file_data.background),
    ];

    if !Path::new(path).exists() {
        std::fs::create_dir(path)
            .map_err(|err| format!("Could not create directory {}: {}", path, err))?;
    } else if !overwrite {
        return Err(format!("Path already exists: {}", path));
    }

    for (file_name, bytes) in file_names {
        let mut file = std::fs::File::create(format!("{}/{}", path, file_name))
            .map_err(|err| format!("Could not create {} in {}: {}", file_name, path, err))?;

        file.write_all(bytes)
            .map_err(|err| format!("Could not save {} in {}: {}", file_name, path, err))?;
    }

    Ok(())
}

fn image_to_png(image: &Image) -> Result<Vec<u8>, String> {
    let mut png_bytes = Vec::new();

    let (width, height) = (image.width(), image.height());
    let mut png_encoder = Encoder::new(&mut png_bytes, width as u32, height as u32);
    png_encoder.set_color(ColorType::RGBA);
    png_encoder.set_depth(BitDepth::Eight);

    let mut png_writer = png_encoder
        .write_header()
        .map_err(|err| format!("Could not write PNG header: {}", err))?;

    png_writer
        .write_image_data(&image.bytes)
        .map_err(|err| format!("Could not write PNG data: {}", err))?;

    drop(png_writer);

    Ok(png_bytes)
}

#[allow(dead_code)]
pub(crate) fn save_grid_to_bin(grid: &WaterGrid) -> Result<(), String> {
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

#[allow(dead_code)]
pub(crate) fn load_grid_from_bin() -> Result<WaterGrid, String> {
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

pub(crate) fn load_water_from_png(bytes: &[u8]) -> Result<WaterGrid, String> {
    let reader = std::io::BufReader::new(bytes);
    let png_decoder = Decoder::new(reader);

    load_water_from_decoder(png_decoder)
}

pub(crate) fn save_water_to_png(grid: &WaterGrid) -> Result<Vec<u8>, String> {
    if cfg!(target_arch = "wasm32") {
        return Err("Saving not yet possible on browsers".to_string());
    }

    let mut bytes = Vec::new();
    let writer = &mut bytes;

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

            let pixel = if let Some(wall_material) = cell.wall_material() {
                match wall_material {
                    WallMaterial::Normal => [255, 255, 255, 255],
                    WallMaterial::Glass => [0, 255, 255, 255],
                }
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

    drop(png_writer);

    Ok(bytes)
}

fn load_water_from_decoder(png_decoder: Decoder<impl Read>) -> Result<WaterGrid, String> {
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
                [0, 255, 255, 255] => cell.make_glass(),
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

fn load_wires_from_yaml(bytes: &[u8], width: usize, height: usize) -> Result<WireGrid, String> {
    let wire_points: Vec<(WireColor, Vec<(usize, usize)>)> = serde_yaml::from_slice(bytes)
        .map_err(|err| format!("Could not load wires from YAML file: {}", err))?;
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
                    wire_grid.make_wire(x, y, color);
                }
            }
        }
    }

    Ok(wire_grid)
}

fn save_wires_to_yaml(wire_grid: &WireGrid) -> Result<Vec<u8>, String> {
    let wire_points = wire_grid.wire_points();

    serde_yaml::to_vec(&wire_points)
        .map_err(|err| format!("Error saving submarine's wire grid: {}", err))
}

fn load_objects_from_yaml(object_bytes: &[u8]) -> Result<Vec<Object>, String> {
    serde_yaml::from_slice(object_bytes)
        .map_err(|err| format!("Error loading objects from yaml: {}", err))
}

fn save_objects_to_yaml(objects: &[Object]) -> Result<Vec<u8>, String> {
    serde_yaml::to_vec(objects).map_err(|err| format!("Error saving objects to yaml: {}", err))
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
