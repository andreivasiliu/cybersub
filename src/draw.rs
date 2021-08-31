use std::{cell::RefCell, collections::HashSet, mem::swap};

use macroquad::{
    camera::{pop_camera_state, push_camera_state, set_default_camera},
    prelude::{
        clear_background, draw_circle, draw_line, draw_rectangle, draw_rectangle_lines, draw_text,
        draw_texture, draw_texture_ex, draw_triangle, get_time, gl_use_default_material,
        gl_use_material, render_target, screen_height, screen_width, set_camera, vec2, Camera2D,
        Color, DrawTextureParams, FilterMode, Image, Rect, Texture2D, Vec2, BLACK, BLANK, DARKBLUE,
        DARKGRAY, DARKGREEN, PURPLE, RED, WHITE, YELLOW,
    },
};

use crate::{Timings, app::{GameSettings, PlacingObject}, game_state::objects::{Object, ObjectType}, game_state::rocks::RockGrid, game_state::sonar::Sonar, game_state::state::{GameState, Navigation, SubmarineState}, game_state::water::WallMaterial, game_state::water::WaterGrid, game_state::wires::{WireColor, WireGrid}, resources::{MutableResources, MutableSubResources, Resources, TurbulenceParticle}, shadows::{Edge, Triangle, add_border_edges, filter_edges_by_direction, filter_edges_by_region, find_shadow_edges, find_shadow_triangles}};

pub(crate) struct DrawSettings {
    pub draw_egui: bool,
    pub draw_sea_dust: bool,
    pub draw_sea_caustics: bool,
    pub draw_rocks: bool,
    pub draw_background: bool,
    pub draw_objects: bool,
    pub draw_walls: bool,
    pub draw_wires: bool,
    pub draw_water: bool,
    pub draw_sonar: bool,
    pub draw_engine_turbulence: bool,
    pub debug_shadows: bool,
}

#[derive(Debug, Default)]
pub(crate) struct Camera {
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: i32,
    pub dragging_from: (f32, f32),
    pub scrolling_from: f32,
    pub pointing_at: (usize, usize),
    pub current_submarine: Option<(i32, i32)>,
}

impl Camera {
    pub fn to_macroquad_camera(&self, submarine: Option<(i32, i32)>) -> Camera2D {
        let zoom = if screen_height() < screen_width() {
            vec2(screen_height() / screen_width(), -1.0) * 1.3
        } else {
            vec2(1.0, -screen_width() / screen_height())
        };

        let mut target = vec2(-self.offset_x as f32, -self.offset_y as f32);

        if let Some(submarine) = submarine {
            target.x -= submarine.0 as f32 / 16.0;
            target.y -= submarine.1 as f32 / 16.0;
        }

        if let Some(submarine) = self.current_submarine {
            target.x += submarine.0 as f32 / 16.0;
            target.y += submarine.1 as f32 / 16.0;
        }

        Camera2D {
            zoom: zoom * (1.5 / 50.0) * self.user_zoom(),
            target,
            ..Default::default()
        }
    }

    fn user_zoom(&self) -> f32 {
        1.0 / (1.0 - self.zoom as f32 / 64.0)
    }
}

fn draw_rect_at(pos: Vec2, size: f32, color: Color) {
    draw_rectangle(pos.x - size, pos.y - size, size * 2.0, size * 2.0, color);
}

pub(crate) fn to_screen_coords(x: usize, y: usize) -> Vec2 {
    vec2(x as f32, y as f32)
}

pub(crate) fn draw_game(
    game_state: &GameState,
    game_settings: &GameSettings,
    timings: &Timings,
    resources: &Resources,
    mutable_resources: &mut MutableResources,
    mutable_sub_resources: &mut [MutableSubResources],
) {
    let GameState {
        rock_grid,
        submarines,
        ..
    } = game_state;
    let GameSettings {
        camera,
        draw_settings,
        ..
    } = game_settings;

    set_camera(&camera.to_macroquad_camera(None));

    if draw_settings.draw_sea_dust || draw_settings.draw_sea_caustics {
        draw_sea(
            camera,
            draw_settings.draw_sea_dust,
            draw_settings.draw_sea_caustics,
            resources,
            rock_grid.size(),
        );
    } else {
        draw_fake_sea(rock_grid.size());
    }

    if draw_settings.draw_engine_turbulence {
        // Draw all external effects of submarines before all submarines, so
        // they don't go over another submarine
        draw_engine_turbulence(
            submarines,
            game_settings.animation_ticks,
            resources,
            mutable_sub_resources,
        );
    }

    if draw_settings.draw_rocks {
        draw_rocks(
            rock_grid,
            &game_state.collisions,
            resources,
            mutable_resources,
        );
    }

    push_camera_state();

    for (sub_index, submarine) in submarines.iter().enumerate() {
        set_camera(&camera.to_macroquad_camera(Some(submarine.navigation.position)));

        let mutable_resources = mutable_sub_resources
            .get_mut(sub_index)
            .expect("All submarines should have their own MutableSubResources instance");

        if draw_settings.draw_background {
            draw_background(mutable_resources);
        }

        if draw_settings.draw_walls {
            draw_walls(
                &submarine.water_grid,
                resources,
                &submarine.collisions,
                mutable_resources,
            );
        }

        if draw_settings.debug_shadows {
            update_shadow_edges(&submarine.water_grid, mutable_resources);
            draw_shadow_edges(&mutable_resources.shadow_edges, mutable_resources.sub_cursor);
        }

        if draw_settings.draw_wires {
            update_wires_texture(&submarine.wire_grid, resources, mutable_resources);
            update_signals_texture(&submarine.wire_grid, mutable_resources);
            draw_wires(&submarine.wire_grid, resources, mutable_resources);
        }

        if draw_settings.draw_objects {
            draw_objects(
                &submarine.objects,
                resources,
                mutable_resources.highlighting_object,
                &game_settings.placing_object,
            );
        }

        if draw_settings.draw_sonar {
            draw_sonar(
                &submarine.objects,
                submarine.water_grid.size(),
                &submarine.sonar,
                &submarine.navigation,
                resources,
                mutable_resources,
            );
        }

        if draw_settings.draw_water {
            draw_water(&submarine.water_grid);
        }
    }

    pop_camera_state();

    if !draw_settings.draw_egui {
        set_default_camera();
        draw_ui_alternative(timings, game_settings.highlighting_settings, resources);
    }
}

pub(crate) fn draw_ui_alternative(
    timings: &Timings,
    highlighting_settings: bool,
    resources: &Resources,
) {
    let frame = if highlighting_settings { 1.0 } else { 0.0 };
    draw_texture_ex(
        resources.settings,
        10.0,
        10.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(20.0, 20.0)),
            source: Some(Rect::new(0.0, 60.0 * frame, 60.0, 60.0)),
            ..Default::default()
        },
    );

    let text = format!(
        "fps: {}, update: {}, layout: {}",
        timings.fps, timings.game_update, timings.game_layout
    );
    draw_text(&text, 40.0, 25.0, 20.0, PURPLE);
}

fn draw_sea(
    camera: &Camera,
    draw_sea_dust: bool,
    draw_sea_caustics: bool,
    resources: &Resources,
    world_size: (usize, usize),
) {
    let (width, height) = world_size;
    let time_offset = vec2(0.1, 1.0) * get_time() as f32 * 0.03;
    let camera_offset = vec2(camera.offset_x, camera.offset_y) / 600.0;
    resources
        .sea_water
        .set_uniform("enable_dust", if draw_sea_dust { 1.0f32 } else { 0.0 });
    resources.sea_water.set_uniform(
        "enable_caustics",
        if draw_sea_caustics { 1.0f32 } else { 0.0 },
    );
    resources.sea_water.set_uniform("time_offset", time_offset);
    resources
        .sea_water
        .set_uniform("camera_offset", camera_offset);
    resources.sea_water.set_uniform("time", get_time() as f32);
    resources.sea_water.set_uniform(
        "world_size",
        vec2((width / 16) as f32, (height / 16) as f32),
    );
    resources.sea_water.set_uniform(
        "sea_dust_size",
        vec2(resources.sea_dust.width(), resources.sea_dust.height()),
    );
    resources
        .sea_water
        .set_texture("sea_dust", resources.sea_dust);

    // The world always starts here.
    let pos = vec2(0.0, 0.0);

    gl_use_material(resources.sea_water);
    draw_rectangle(
        pos.x,
        pos.y,
        (width * 16) as f32,
        (height * 16) as f32,
        WHITE,
    );
    gl_use_default_material();
}

fn draw_fake_sea(world_size: (usize, usize)) {
    let (width, height) = world_size;

    draw_rectangle(
        0.0,
        0.0,
        (width * 16) as f32,
        (height * 16) as f32,
        Color::new(0.0235, 0.0235, 0.1255, 1.0),
    );
}

fn draw_background(mutable_resources: &MutableSubResources) {
    let top_left = to_screen_coords(0, 0);
    draw_texture(
        mutable_resources.sub_background,
        top_left.x,
        top_left.y,
        WHITE,
    );
}

fn draw_walls(
    grid: &WaterGrid,
    resources: &Resources,
    collisions: &[(usize, usize)],
    mutable_resources: &mut MutableSubResources,
) {
    let (width, height) = grid.size();

    let texture = mutable_resources.sub_walls;
    let (old_width, old_height) = (texture.width() as usize, texture.height() as usize);

    if mutable_resources.walls_updated || width != old_width || height != old_height {
        mutable_resources.walls_updated = false;

        let mut image = Image::gen_image_color(width as u16, height as u16, BLANK);

        for y in 0..height {
            for x in 0..width {
                let cell = grid.cell(x, y);

                if let Some(wall_material) = cell.wall_material() {
                    let color = match wall_material {
                        WallMaterial::Normal => WHITE,
                        WallMaterial::Glass => Color::new(0.0, 1.0, 1.0, 1.0),
                    };
                    image.set_pixel(x as u32, y as u32, color);
                }
            }
        }

        if old_width != width || old_height != height {
            mutable_resources.sub_walls.delete();
            mutable_resources.sub_walls = Texture2D::from_image(&image);
            mutable_resources.sub_walls.set_filter(FilterMode::Nearest);
        } else {
            mutable_resources.sub_walls.update(&image);
        }
    }

    resources
        .wall_material
        .set_texture("wall_texture", resources.wall);
    resources
        .wall_material
        .set_texture("glass_texture", resources.glass);
    resources
        .wall_material
        .set_texture("walls", mutable_resources.sub_walls);
    resources
        .wall_material
        .set_uniform("walls_size", vec2(width as f32, height as f32));
    gl_use_material(resources.wall_material);

    let pos = to_screen_coords(0, 0);

    draw_texture_ex(
        mutable_resources.sub_walls,
        pos.x,
        pos.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width as f32, height as f32)),
            // source: Some(Rect::new(0.0, 0.0, width as f32 * 5.0, height as f32 * 5.0)),
            ..Default::default()
        },
    );

    gl_use_default_material();

    for &(x, y) in collisions {
        let pos = to_screen_coords(x, y) + vec2(0.5, 0.5);
        draw_circle(pos.x, pos.y, 0.5, RED);
    }
}

fn draw_water(grid: &WaterGrid) {
    let (width, height) = grid.size();

    for i in 0..width {
        for j in 0..height {
            let cell = grid.cell(i, j);

            if !cell.is_inside() {
                continue;
            }

            let pos = to_screen_coords(i, j) + vec2(0.5, 0.5);
            let level = grid.cell(i, j).amount_filled();
            let overlevel = grid.cell(i, j).amount_overfilled();
            let velocity = grid.cell(i, j).velocity();

            let level = if level != 0.0 && level < 0.5 {
                0.5
            } else {
                level
            };

            let size = 0.5;

            let transparent_blue = Color::new(0.40, 0.75, 1.00, 0.75);

            if level > 0.0 {
                draw_rect_at(pos, size * level, transparent_blue);
                draw_rect_at(pos, size * overlevel, DARKBLUE);

                let velocity = vec2(velocity.0, velocity.1).normalize_or_zero() * 0.35;

                if velocity != vec2(0.0, 0.0) {
                    draw_line(
                        pos.x,
                        pos.y,
                        pos.x + velocity.x,
                        pos.y + velocity.y,
                        0.1,
                        BLACK,
                    );
                }
            }
        }
    }
}

fn update_shadow_edges(water_grid: &WaterGrid, mutable_resources: &mut MutableSubResources) {
    if mutable_resources.shadow_edges_updated {
        mutable_resources.shadow_edges = find_shadow_edges(water_grid);
        mutable_resources.shadow_edges_updated = false;
    }
}

fn draw_shadow_edges(edges: &[Edge], cursor: Option<(f32, f32)>) -> () {
    for edge in edges {
        let (start, end) = edge.line();

        draw_line(start.x, start.y, end.x, end.y, 0.1, PURPLE);
    }

    let range = 30.0;

    if let Some(cursor) = cursor {
        let cursor: Vec2 = cursor.into();

        let edges = filter_edges_by_region(edges, cursor, range);

        for edge in &edges {
            let (start, end) = edge.line();

            draw_line(start.x, start.y, end.x, end.y, 0.1, RED);

            let gray = Color::new(0.4, 0.0, 0.0, 0.2);

            draw_line(cursor.x, cursor.y, start.x, start.y, 0.1, gray);
            draw_line(cursor.x, cursor.y, end.x, end.y, 0.1, gray);
        }

        let mut edges = filter_edges_by_direction(edges, cursor);

        for edge in &edges {
            let (start, end) = edge.line();

            draw_line(
                start.x as f32,
                start.y as f32,
                end.x as f32,
                end.y as f32,
                0.1,
                YELLOW,
            );

            let yellow = Color::new(0.0, 0.4, 0.0, 0.2);

            draw_line(cursor.x, cursor.y, start.x, start.y, 0.1, yellow);
            draw_line(cursor.x, cursor.y, end.x, end.y, 0.1, yellow);
        }

        add_border_edges(&mut edges, cursor, range);
        let (triangles, points) = find_shadow_triangles(edges, cursor, range);

        for Triangle(p1, p2, p3) in triangles {
            let purple = Color::new(0.2, 0.0, 0.2, 0.2);

            draw_triangle(p1, p2, p3, purple);
        }

        for (point_index, point) in points.iter().enumerate() {
            // Show order of points by gradually making them yellower
            let green = 1.0 * point_index as f32 / points.len() as f32;
            let color = Color::new(1.0, green, 0.0, 1.0);
            draw_circle(point.x, point.y, 1.0, color);
        }
    }
}

fn update_wires_texture(
    grid: &WireGrid,
    resources: &Resources,
    mutable_resources: &mut MutableSubResources,
) {
    let old_size = (
        mutable_resources.sub_wires.texture.width() as usize,
        mutable_resources.sub_wires.texture.height() as usize,
    );

    // Wire cells are 6x6 pixels each
    let new_size = (grid.size().0 * 6, grid.size().1 * 6);

    if !mutable_resources.wires_updated && old_size == new_size {
        return;
    }

    if old_size != new_size {
        mutable_resources.sub_wires.delete();
        mutable_resources.sub_wires = render_target(new_size.0 as u32, new_size.1 as u32);
        mutable_resources
            .sub_wires
            .texture
            .set_filter(FilterMode::Nearest);
    }

    mutable_resources.wires_updated = false;

    let (width, height) = grid.size();
    let grid_size = vec2(width as f32, height as f32);

    // Draw the wires' special colors to an offscreen texture
    push_camera_state();

    set_camera(&Camera2D {
        // target: sonar_size / 2.0,
        render_target: Some(mutable_resources.sub_wires),
        zoom: 2.0 / grid_size,
        offset: vec2(-1.0, -1.0),
        ..Default::default()
    });

    clear_background(BLANK);

    for x in 0..width {
        for y in 0..height {
            let pos = vec2(x as f32, y as f32);

            let cell = grid.cell(x, y);

            let colors = &[
                WireColor::Purple,
                WireColor::Brown,
                WireColor::Blue,
                WireColor::Green,
            ];

            for wire_color in colors {
                if !cell.value(*wire_color).connected() {
                    continue;
                }

                let has_neighbours = grid.has_neighbours(*wire_color, x, y);

                let wire_color_frames = 4;
                let wire_color_frame = *wire_color as u16;

                let wire_type_frames = 7;
                let wire_type_frame = match has_neighbours {
                    // [down, right, up, left]
                    [true, false, true, false] => 0,
                    [false, false, true, false] => 0,
                    [true, false, false, false] => 0,
                    [false, true, false, true] => 1,
                    [false, false, false, true] => 1,
                    [false, true, false, false] => 1,
                    [true, true, false, false] => 2,
                    [true, false, false, true] => 3,
                    [false, false, true, true] => 4,
                    [false, true, true, false] => 5,
                    _ => 6,
                };

                // The wires texture is vertically split into frames by wire
                // direction, and horizontally split by wire color
                let frame_width = (resources.wires.width() as u16 / wire_color_frames) as f32;
                let frame_height = (resources.wires.height() as u16 / wire_type_frames) as f32;
                let frame_x = (frame_width as u16 * wire_color_frame) as f32;
                let frame_y = (frame_height as u16 * wire_type_frame) as f32;

                draw_texture_ex(
                    resources.wires,
                    pos.x,
                    pos.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(1.0, 1.0)),
                        source: Some(Rect::new(frame_x, frame_y, frame_width, frame_height)),
                        ..Default::default()
                    },
                );
            }
        }
    }

    draw_circle(1.0, 1.0, 15.0, RED);

    pop_camera_state();
}

fn update_signals_texture(grid: &WireGrid, mutable_resources: &mut MutableSubResources) {
    let old_size = (
        mutable_resources.sub_signals.width() as usize,
        mutable_resources.sub_signals.height() as usize,
    );

    if !mutable_resources.signals_updated && old_size == grid.size() {
        return;
    }

    mutable_resources.signals_updated = false;

    let (width, height) = grid.size();

    if old_size != grid.size() {
        mutable_resources.sub_signals_image =
            Image::gen_image_color(width as u16, height as u16, BLANK);
    }

    let colors = &[
        WireColor::Purple,
        WireColor::Brown,
        WireColor::Blue,
        WireColor::Green,
    ];

    let image = &mut mutable_resources.sub_signals_image;

    for y in 0..height {
        for x in 0..width {
            let cell = grid.cell(x, y);

            for wire_color in colors {
                let signal = cell.value(*wire_color).signal();
                let brightness = (signal as f32 / 256.0 + 0.2).clamp(0.0, 1.0);

                if signal > 0 {
                    let mut color = image.get_pixel(x as u32, y as u32);

                    // Encode signal brightness as one of the RGBA components
                    // This will be used by a fragment shader to light up wires of that
                    // particular color.
                    match wire_color {
                        WireColor::Purple => color.r = brightness,
                        WireColor::Brown => color.g = brightness,
                        WireColor::Blue => color.b = brightness,
                        WireColor::Green => color.a = brightness,
                    };

                    image.set_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    if old_size != grid.size() {
        mutable_resources.sub_signals.delete();
        mutable_resources.sub_signals = Texture2D::from_image(image);
    } else {
        mutable_resources.sub_signals.update(image);
    }
}

fn draw_wires(grid: &WireGrid, resources: &Resources, mutable_resources: &MutableSubResources) {
    let (width, height) = grid.size();

    let pos = to_screen_coords(0, 0);
    let grid_size = vec2(width as f32, height as f32);

    resources
        .wire_material
        .set_texture("sub_wires", mutable_resources.sub_wires.texture);
    resources
        .wire_material
        .set_texture("sub_signals", mutable_resources.sub_signals);
    resources.wire_material.set_uniform("grid_size", grid_size);

    gl_use_material(resources.wire_material);

    draw_texture_ex(
        mutable_resources.sub_wires.texture,
        pos.x,
        pos.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(grid_size),
            ..Default::default()
        },
    );

    gl_use_default_material();
}

pub(crate) fn object_rect(object: &Object) -> Rect {
    let pos = to_screen_coords(object.position.0 as usize, object.position.1 as usize);

    let size = object_size(&object.object_type);
    let size = vec2(size.0 as f32, size.1 as f32);

    Rect::new(pos.x + 1.0, pos.y + 1.0, size.x, size.y)
}

pub(crate) fn object_size(object_type: &ObjectType) -> (usize, usize) {
    match object_type {
        ObjectType::Door { .. } => (22, 5),
        ObjectType::VerticalDoor { .. } => (5, 17),
        ObjectType::Reactor { .. } => (32, 17),
        ObjectType::Lamp => (5, 4),
        ObjectType::Gauge { .. } => (7, 7),
        ObjectType::SmallPump { .. } => (9, 7),
        ObjectType::LargePump { .. } => (30, 18),
        ObjectType::JunctionBox => (6, 8),
        ObjectType::NavController { .. } => (9, 15),
        ObjectType::Sonar { .. } => (19, 17),
        ObjectType::Engine { .. } => (37, 20),
        ObjectType::Battery { .. } => (8, 10),
    }
}

fn object_frames(object_type: &ObjectType) -> u16 {
    match object_type {
        ObjectType::Door { .. } => 8,
        ObjectType::VerticalDoor { .. } => 9,
        ObjectType::Reactor { .. } => 2,
        ObjectType::Lamp => 2,
        ObjectType::Gauge { .. } => 5,
        ObjectType::SmallPump { .. } => 4,
        ObjectType::LargePump { .. } => 4,
        ObjectType::JunctionBox => 1,
        ObjectType::NavController { .. } => 6,
        ObjectType::Sonar { .. } => 2,
        ObjectType::Engine { .. } => 24,
        ObjectType::Battery { .. } => 8,
    }
}

fn object_texture(object_type: &ObjectType, resources: &Resources) -> Texture2D {
    match object_type {
        ObjectType::Door { .. } => resources.hatch,
        ObjectType::VerticalDoor { .. } => resources.door,
        ObjectType::Reactor { .. } => resources.reactor,
        ObjectType::Lamp => resources.lamp,
        ObjectType::Gauge { .. } => resources.gauge,
        ObjectType::SmallPump { .. } => resources.small_pump,
        ObjectType::LargePump { .. } => resources.large_pump,
        ObjectType::JunctionBox => resources.junction_box,
        ObjectType::NavController { .. } => resources.nav_controller,
        ObjectType::Sonar { .. } => resources.sonar,
        ObjectType::Engine { .. } => resources.engine,
        ObjectType::Battery { .. } => resources.battery,
    }
}

fn draw_objects(
    objects: &[Object],
    resources: &Resources,
    highlighting_object: Option<usize>,
    placing_object: &Option<PlacingObject>,
) {
    let draw_object = |object, highlight, shadow| {
        let draw_rect = object_rect(object);

        let texture = object_texture(&object.object_type, resources);

        // Textures are vertically split into equally-sized animation frames
        let frame_width = texture.width();
        let frame_height = (texture.height() as u16 / object_frames(&object.object_type)) as f32;
        let frame_x = 0.0;
        let frame_y = (frame_height as u16 * object.current_frame) as f32;

        draw_texture_ex(
            texture,
            draw_rect.x,
            draw_rect.y,
            if shadow {
                Color::new(0.5, 0.5, 1.0, 0.5)
            } else {
                WHITE
            },
            DrawTextureParams {
                dest_size: Some(draw_rect.size()),
                source: Some(Rect::new(frame_x, frame_y, frame_width, frame_height)),
                ..Default::default()
            },
        );

        if highlight {
            let texture_resolution = vec2(texture.width(), texture.height());
            resources
                .hover_highlight
                .set_uniform("input_resolution", texture_resolution);
            resources.hover_highlight.set_uniform("frame_y", frame_y);
            resources
                .hover_highlight
                .set_uniform("frame_height", frame_height);
            resources
                .hover_highlight
                .set_texture("input_texture", texture);
            gl_use_material(resources.hover_highlight);
            let r = draw_rect;
            draw_rectangle(r.x, r.y, r.w, r.h, DARKBLUE);
            gl_use_default_material();
        }
    };

    for (obj_id, object) in objects.iter().enumerate() {
        let highlight = highlighting_object == Some(obj_id);
        let shadow = false;
        draw_object(object, highlight, shadow);
    }

    if let Some(PlacingObject {
        position: Some((x, y)),
        object_type,
        ..
    }) = placing_object
    {
        let object = Object {
            object_type: object_type.clone(),
            position: (*x as u32, *y as u32),
            current_frame: 0,
        };

        let highlight = false;
        let shadow = true;
        draw_object(&object, highlight, shadow);
    }
}

fn update_rocks_texture(grid: &RockGrid, mutable_resources: &mut MutableResources) {
    if mutable_resources.sea_rocks_updated {
        return;
    }

    let (width, height) = grid.size();

    let mut image = Image::gen_image_color(width as u16, height as u16, BLANK);

    for y in 0..height {
        for x in 0..width {
            let cell = grid.cell(x, y);

            if !cell.is_wall() {
                continue;
            }

            // Encode the tile as a color for the shader to use.
            let frame_offset = cell.rock_type() as usize;
            image.set_pixel(
                x as u32,
                y as u32,
                Color::new(frame_offset as f32 / 16.0, 0.0, 0.0, 1.0),
            );
        }
    }

    let img_width = mutable_resources.sea_rocks.width() as usize;
    let img_height = mutable_resources.sea_rocks.height() as usize;

    if img_width != width || img_height != height {
        mutable_resources.sea_rocks.delete();
        mutable_resources.sea_rocks = Texture2D::from_image(&image);
        mutable_resources.sea_rocks.set_filter(FilterMode::Nearest);
    } else {
        mutable_resources.sea_rocks.update(&image);
    }

    mutable_resources.sea_rocks_updated = true;
}

fn draw_engine_turbulence(
    submarines: &[SubmarineState],
    animation_ticks: u32,
    resources: &Resources,
    mutable_sub_resources: &mut [MutableSubResources],
) {
    for (sub_index, submarine) in submarines.iter().enumerate() {
        for object in &submarine.objects {
            if let ObjectType::Engine { speed, .. } = &object.object_type {
                let mutable_resources = mutable_sub_resources
                    .get_mut(sub_index)
                    .expect("All submarines should have their own MutableSubResources instance");

                let pos = vec2(
                    submarine.navigation.position.0 as f32 / 16.0 + object.position.0 as f32,
                    submarine.navigation.position.1 as f32 / 16.0 + object.position.1 as f32,
                ) + vec2(0.0, 2.0);

                for _tick in 0..animation_ticks {
                    if *speed != 0 {
                        for _new_particle in 0..5 {
                            let frame = (random() * 4.9) as u8;
                            mutable_resources
                                .turbulence_particles
                                .push(TurbulenceParticle {
                                    position: (pos.x + random() * 3.0, pos.y + random() * 6.0),
                                    frame,
                                    speed: *speed as f32 * (random() / 4.0 + 0.75),
                                    life: (128.0 * (random() / 2.0 + 0.5)) as u8,
                                });
                        }
                    }

                    for particle in mutable_resources.turbulence_particles.iter_mut() {
                        particle.position.0 -= (0.5 * particle.life as f32 / 32.0
                            * (particle.frame + 30) as f32
                            / 32.0)
                            * (particle.speed as f32 / 64.0);
                        particle.position.1 += 0.001;

                        particle.life -= 1;
                    }
                    mutable_resources
                        .turbulence_particles
                        .retain(|particle| particle.life != 0);
                }

                for particle in mutable_resources.turbulence_particles.iter_mut() {
                    let (x, y) = particle.position;

                    draw_texture_ex(
                        resources.turbulence,
                        x,
                        y,
                        Color::new(1.0, 1.0, 1.0, particle.life as f32 / 128.0),
                        DrawTextureParams {
                            dest_size: Some(vec2(5.0, 5.0)),
                            source: Some(Rect::new(
                                0.0,
                                128.0 * particle.frame as f32,
                                128.0,
                                128.0,
                            )),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }
}

fn draw_rocks(
    grid: &RockGrid,
    collisions: &[(usize, usize)],
    resources: &Resources,
    mutable_resources: &mut MutableResources,
) {
    update_rocks_texture(grid, mutable_resources);

    let (width, height) = grid.size();

    resources
        .rock_material
        .set_texture("rocks_texture", resources.rocks);
    resources
        .rock_material
        .set_texture("sea_rocks", mutable_resources.sea_rocks);
    resources
        .rock_material
        .set_uniform("sea_rocks_size", vec2(width as f32, height as f32));
    gl_use_material(resources.rock_material);

    // The world always starts here.
    let pos = vec2(0.0, 0.0);

    draw_texture_ex(
        mutable_resources.sea_rocks,
        pos.x,
        pos.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width as f32, height as f32) * 16.0),
            ..Default::default()
        },
    );

    gl_use_default_material();

    let collision_set: HashSet<_> = collisions.iter().collect();
    for &(x, y) in collision_set {
        let pos = vec2(x as f32 + 0.5, y as f32 + 0.5) * 16.0;
        draw_rect_at(pos, 8.0, Color::new(0.78, 0.48, 1.00, 0.2));
    }
}

fn draw_sonar(
    objects: &[Object],
    grid_size: (usize, usize),
    sonar: &Sonar,
    navigation: &Navigation,
    resources: &Resources,
    mutable_resources: &mut MutableSubResources,
) {
    let resolution = 16.0;

    // 13 cells, 6 pixels each
    let sonar_size = vec2(13.0 * resolution, 13.0 * resolution);
    let (width, height) = (13 * resolution as u32, 13 * resolution as u32);

    let old_size = vec2(
        mutable_resources.old_sonar_target.texture.width(),
        mutable_resources.old_sonar_target.texture.height(),
    );

    if sonar_size != old_size {
        let mut targets = [
            &mut mutable_resources.old_sonar_target,
            &mut mutable_resources.new_sonar_target,
        ];

        for target in &mut targets {
            target.delete();
            **target = render_target(width, height);
            target.texture.set_filter(FilterMode::Nearest);
        }
    }

    // Draw the sonar's contents to an offscreen texture
    if mutable_resources.sonar_updated || sonar_size != old_size {
        mutable_resources.sonar_updated = false;

        push_camera_state();

        swap(
            &mut mutable_resources.old_sonar_target,
            &mut mutable_resources.new_sonar_target,
        );

        set_camera(&Camera2D {
            // target: sonar_size / 2.0,
            render_target: Some(mutable_resources.new_sonar_target),
            zoom: 1.0 / sonar_size,
            ..Default::default()
        });

        clear_background(BLANK);

        let sonar_radius_squared = (sonar_size.x * sonar_size.x) * 0.95;

        // Rock edges up to 75 rock-cells away
        for (x, y) in sonar.visible_edge_cells() {
            // A rock-cell is 16 bigger than a normal one
            let pos = vec2(
                -*x as f32 * 16.0 * resolution / 75.0,
                -*y as f32 * 16.0 * resolution / 75.0,
            );

            if pos.length_squared() >= sonar_radius_squared {
                continue;
            }

            // Red encodes brightness
            let red = 1.0;
            // Green encodes distance; shared by the shadows, so that the whole
            // point and all its shadows appear simultaneously
            let green = pos.length() / resolution / 16.0;

            draw_circle(
                pos.x,
                pos.y,
                resolution / 3.0,
                Color::new(red, green, 0.0, 1.0),
            );

            let normal = pos.normalize_or_zero();

            for shadow in 1..5 {
                let pos = pos + normal * (shadow as f32) * resolution * 0.7;
                let red = (1.0 - shadow as f32 / 5.0) * 0.7;
                if pos.length_squared() < sonar_radius_squared {
                    draw_circle(
                        pos.x,
                        pos.y,
                        resolution / 4.0,
                        Color::new(red, green, 0.0, 1.0),
                    );
                }
            }
        }

        pop_camera_state();
    }

    // Draw the offscreen texture to all sonar objects in the current submarine
    let (width, height) = grid_size;
    let texture = mutable_resources.new_sonar_target.texture;

    for (obj_index, object) in objects.iter().enumerate() {
        let sonar_target = match object.active_sonar_target() {
            Some(target) => target,
            None => continue,
        };

        let draw_rect = object_rect(object);
        let pos = draw_rect.point() + vec2(4.0, 2.0);

        resources
            .sonar_material
            .set_texture("new_sonar_texture", texture);
        resources.sonar_material.set_texture(
            "old_sonar_texture",
            mutable_resources.old_sonar_target.texture,
        );
        resources
            .sonar_material
            .set_uniform("sonar_texture_size", sonar_size);
        resources.sonar_material.set_uniform("pulse", sonar.pulse());

        gl_use_material(resources.sonar_material);

        draw_texture_ex(
            texture,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(11.0, 11.0)),
                ..Default::default()
            },
        );

        gl_use_default_material();

        let center = pos + vec2(5.5, 5.5);

        // Mini representation of the submarine
        let sub_size = vec2(width as f32, height as f32) / 16.0 / resolution;
        let sub_pos = center - sub_size / 2.0;
        let sub_color = Color::new(0.40, 0.75, 1.00, 0.50);

        draw_texture_ex(
            mutable_resources.sub_background,
            sub_pos.x,
            sub_pos.y,
            sub_color,
            DrawTextureParams {
                dest_size: Some(sub_size),
                ..Default::default()
            },
        );

        // Cursor (last point where mouse was)
        let cursor = match mutable_resources.sonar_cursor {
            Some((obj, cursor)) if obj == obj_index => Some(cursor),
            Some(_) | None => None,
        };
        if let Some(cursor) = cursor {
            let cursor = center + cursor.into();
            draw_line(
                cursor.x - 0.2,
                cursor.y,
                cursor.x + 0.2,
                cursor.y,
                0.05,
                DARKGREEN,
            );
            draw_line(
                cursor.x,
                cursor.y - 0.2,
                cursor.x,
                cursor.y + 0.2,
                0.05,
                DARKGREEN,
            );
        }

        // Navigation target
        if let Some(sonar_target) = sonar_target {
            let target = vec2(
                (sonar_target.0 as i32 - navigation.position.0) as f32,
                (sonar_target.1 as i32 - navigation.position.1) as f32,
            );
            let target = center + (target / 16.0 / 16.0 / 75.0 * 6.0).clamp_length_max(5.5);
            draw_line(center.x, center.y, target.x, target.y, 0.05, DARKGREEN);
            draw_rectangle_lines(target.x - 0.1, target.y - 0.1, 0.2, 0.2, 0.05, DARKGREEN);
        }

        // Current velocity
        let speed = vec2(
            navigation.speed.0 as f32 / 1024.0,
            navigation.speed.1 as f32 / 1024.0,
        );
        let speed_line = center + speed * 1.0;
        draw_line(
            center.x,
            center.y,
            speed_line.x,
            speed_line.y,
            0.05,
            DARKGRAY,
        );
    }
}

/// Generate a random number from 0.0 to 1.0 using Lehmer’s generator
fn random() -> f32 {
    thread_local! {
        static RNG_STATE: RefCell<u128> = RefCell::new(123);
    }

    let mut number = 0;

    RNG_STATE.with(|local| {
        let mut state = local.borrow_mut();
        *state *= 0xda942042e4dd58b5;
        number = *state >> 64;
    });

    number as f32 / u64::MAX as f32
}
