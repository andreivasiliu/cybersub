use std::{io::Write, path::Path};

use flate2::read::GzDecoder;
use macroquad::prelude::{Image, ImageFormat, BLACK};
use png::{BitDepth, ColorType, Decoder, Encoder};

use crate::{
    game_state::objects::Object,
    game_state::rocks::{RockGrid, RockType},
    game_state::state::SubmarineState,
    game_state::{
        objects::ObjectTemplate,
        wires::{WireColor, WireGrid, WirePoints},
    },
    game_state::{
        state::SubmarineTemplate,
        water::{CellTemplate, WallMaterial, WaterGrid},
    },
    resources::MutableSubResources,
};

pub struct SubmarineFileData {
    pub water_grid: Vec<u8>,
    pub background: Vec<u8>,
    pub objects: Vec<u8>,
    pub wires: Vec<u8>,
}

pub(crate) fn load_template_from_data(
    file_data: SubmarineFileData,
) -> Result<SubmarineTemplate, String> {
    let water_cells = load_water_cells_from_png(&file_data.water_grid)?;
    let wire_points = load_wire_points_from_yaml(&file_data.wires)?;
    let objects = load_objects_from_yaml(&file_data.objects)?;
    let background_image =
        Image::from_file_with_format(&file_data.background, Some(ImageFormat::Png));

    let (width, height, water_cells) = water_cells;

    if background_image.width() != width || background_image.height() != height {
        return Err("Background size does not correspond to water grid size.".to_string());
    }

    Ok(SubmarineTemplate {
        size: (width, height),
        water_cells,
        background_pixels: background_image.bytes,
        objects,
        wire_points,
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
    png_encoder.set_color(ColorType::Rgba);
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

pub(crate) fn save_water_to_png(grid: &WaterGrid) -> Result<Vec<u8>, String> {
    if cfg!(target_arch = "wasm32") {
        return Err("Saving not yet possible on browsers".to_string());
    }

    let mut bytes = Vec::new();
    let writer = &mut bytes;

    let (width, height) = grid.size();
    let mut png_encoder = Encoder::new(writer, width as u32, height as u32);
    png_encoder.set_color(ColorType::Rgba);
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
                    WallMaterial::Glass => [255, 0, 255, 255],
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

fn load_water_cells_from_png(
    png_bytes: &[u8],
) -> Result<(usize, usize, Vec<CellTemplate>), String> {
    let reader = std::io::BufReader::new(png_bytes);
    let png_decoder = Decoder::new(reader);

    let mut png_reader = png_decoder
        .read_info()
        .map_err(|err| format!("Could not read PNG header: {}", err))?;

    let png_info = png_reader.info();

    let (width, height) = (png_info.width as usize, png_info.height as usize);
    let mut water_template = vec![CellTemplate::Sea; width * height];

    if png_info.bit_depth != BitDepth::Eight {
        return Err("PNG must be RGBA with 8 bits per channel".to_owned());
    }

    for y in 0..height {
        let row = png_reader
            .next_row()
            .map_err(|err| format!("Error reading PNG row: {}", err))?
            .expect("Expected row count to equal PNG height");

        let data = row.data();

        for x in 0..width {
            let cell = &mut water_template[y * width + x];

            *cell = match data[x * 4..x * 4 + 4] {
                [0, 0, 255, 255] => CellTemplate::Sea,
                [0, 255, 255, 255] => CellTemplate::Water,
                [255, 255, 255, 255] => CellTemplate::Wall,
                [255, 0, 255, 255] => CellTemplate::Glass,
                [0, 0, 0, 0] => CellTemplate::Inside,
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

    Ok((width, height, water_template))
}

fn load_wire_points_from_yaml(bytes: &[u8]) -> Result<Vec<WirePoints>, String> {
    let wire_points: Vec<(WireColor, Vec<(usize, usize)>)> = serde_yaml::from_slice(bytes)
        .map_err(|err| format!("Could not load wires from YAML file: {}", err))?;

    Ok(wire_points)
}

fn save_wires_to_yaml(wire_grid: &WireGrid) -> Result<Vec<u8>, String> {
    let wire_points = wire_grid.wire_points();

    serde_yaml::to_vec(&wire_points)
        .map_err(|err| format!("Error saving submarine's wire grid: {}", err))
}

fn load_objects_from_yaml(object_bytes: &[u8]) -> Result<Vec<Object>, String> {
    let objects: Vec<ObjectTemplate> = serde_yaml::from_slice(object_bytes)
        .map_err(|err| format!("Error loading objects from yaml: {}", err))?;

    Ok(objects.iter().map(|object| object.to_object()).collect())
}

fn save_objects_to_yaml(objects: &[Object]) -> Result<Vec<u8>, String> {
    let objects: Vec<ObjectTemplate> = objects
        .iter()
        .map(|object| ObjectTemplate::from_object(object))
        .collect();

    serde_yaml::to_vec(&objects).map_err(|err| format!("Error saving objects to yaml: {}", err))
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

pub(crate) fn pixels_to_image(width: usize, height: usize, pixels: &[u8]) -> Image {
    let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);

    let img_bytes = image.get_image_data_mut();

    for y in 0..height {
        for x in 0..width {
            let pixel = &pixels[y * width * 4 + x * 4..y * width * 4 + x * 4 + 4];
            let img_pixel = &mut img_bytes[y * width + x];

            img_pixel[..4].clone_from_slice(&pixel[..4]);
        }
    }

    image
}
