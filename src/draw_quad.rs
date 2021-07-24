use macroquad::{
    camera::set_camera,
    prelude::{
        get_internal_gl, vec2, Color, DrawMode, InternalGlContext, Vec2, Vertex, BLACK, DARKBLUE,
        GRAY, SKYBLUE, WHITE,
    },
};

use crate::{
    draw::{to_screen_coords, Camera},
    water::WaterGrid,
};

// Experimental low-level drawing, since performance on Firefox is disappointing.
// Spoiler: it's actually worse. I think I need to go even lower level with miniquad.

#[derive(Default)]
struct Batch {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Batch {
    fn draw_rect_at(&mut self, pos: Vec2, size: f32, color: Color) {
        let x1 = pos.x - size;
        let x2 = pos.x + size;
        let y1 = pos.y - size;
        let y2 = pos.y + size;

        let i = self.vertices.len() as u16;

        self.vertices.extend_from_slice(&[
            Vertex::new(x1, y1, 0.0, 0.0, 0.0, color),
            Vertex::new(x2, y1, 0.0, 1.0, 0.0, color),
            Vertex::new(x2, y2, 0.0, 1.0, 1.0, color),
            Vertex::new(x1, y2, 0.0, 0.0, 1.0, color),
        ]);

        self.indices
            .extend_from_slice(&[i + 0, i + 1, i + 2, i + 0, i + 2, i + 3]);
    }

    fn maybe_flush(&mut self, gl: &mut InternalGlContext<'_>) {
        if self.indices.len() > 256 || self.vertices.len() > 256 {
            self.flush(gl);
        }
    }

    fn flush(&mut self, gl: &mut InternalGlContext<'_>) {
        gl.quad_gl.texture(None);
        gl.quad_gl.draw_mode(DrawMode::Triangles);
        gl.quad_gl.geometry(&self.vertices, &self.indices);
        self.vertices.clear();
        self.indices.clear();
    }
}

pub(crate) fn draw_quad_game(grid: &WaterGrid, camera: &Camera) {
    let camera = camera.to_macroquad_camera();
    set_camera(&camera);

    let mut gl = unsafe { get_internal_gl() };

    gl.flush();

    let mut batch = Batch::default();

    let (width, height) = grid.size();

    for i in 0..width {
        for j in 0..height {
            let pos = to_screen_coords(i, j, width, height);

            let level = grid.cell(i, j).amount_filled();
            let overlevel = grid.cell(i, j).amount_overfilled();
            // let velocity = grid.cell(i, j).velocity();

            let level = if level != 0.0 && level < 0.5 {
                0.5
            } else {
                level
            };

            let size = 0.35;

            batch.draw_rect_at(pos, size * 1.05, GRAY);
            batch.draw_rect_at(pos, size, BLACK);

            if grid.cell(i, j).is_wall() {
                batch.draw_rect_at(pos, size, GRAY);
            } else {
                batch.draw_rect_at(pos, size * level, SKYBLUE);
                batch.draw_rect_at(pos, size * overlevel, DARKBLUE);
            }

            batch.maybe_flush(&mut gl);
        }
    }

    batch.draw_rect_at(vec2(0.0, 0.0), 0.1, WHITE);
    batch.flush(&mut gl);
}
