use eframe::{
    egui::{
        self, emath::RectTransform, pos2, vec2, Color32, Frame, Rect, Sense, Shape, Stroke, Vec2,
    },
    epi,
};

use crate::water::WaterGrid;

pub struct CyberSubApp {
    // Example stuff:
    label: String,
    grid: WaterGrid,
    show_total_water: bool,
    enable_gravity: bool,
    enable_inertia: bool,
    last_update: Option<f64>,
}

impl Default for CyberSubApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello me!".to_owned(),
            grid: WaterGrid::new(60, 40),
            show_total_water: false,
            enable_gravity: true,
            enable_inertia: true,
            last_update: None,
        }
    }
}

impl epi::App for CyberSubApp {
    fn name(&self) -> &str {
        "CyberSub"
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            label,
            grid,
            show_total_water,
            enable_gravity,
            enable_inertia,
            last_update,
        } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Show total water").clicked() {
                        *show_total_water = !*show_total_water;
                    }
                    if ui.button("Toggle gravity").clicked() {
                        *enable_gravity = !*enable_gravity;
                    }
                    if ui.button("Toggle inertia").clicked() {
                        *enable_inertia = !*enable_inertia;
                    }
                    if ui.button("Clear water").clicked() {
                        grid.clear();
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
                egui::warn_if_debug_build(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });
            Frame::dark_canvas(ui.style()).show(ui, |ui| {
                ui.ctx().request_repaint();

                let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::drag());

                let click_pos = if response.dragged() {
                    response.hover_pos()
                } else {
                    None
                };

                let to_screen = RectTransform::from_to(
                    Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0),
                    painter.clip_rect(),
                );

                let mut shapes = Vec::with_capacity(10 * 10);

                let width = 60;
                let height = 40;

                for i in 0..width {
                    for j in 0..height {
                        let pos = pos2(
                            i as f32 / width as f32 + 1.0 / width as f32 / 2.0,
                            j as f32 / height as f32 + 1.0 / height as f32 / 2.0,
                        );
                        let level = grid.cell(i, j).amount_filled();
                        let overlevel = grid.cell(i, j).amount_overfilled();
                        let velocity: Vec2 = grid.cell(i, j).velocity().into();

                        let level = if level != 0.0 && level < 0.5 {
                            0.5
                        } else {
                            level
                        };

                        let rect1 = Rect::from_center_size(to_screen * pos, vec2(10.0, 10.0));
                        let rect2 =
                            Rect::from_center_size(to_screen * pos, vec2(10.0, 10.0) * level);
                        let rect3 =
                            Rect::from_center_size(to_screen * pos, vec2(10.0, 10.0) * overlevel);

                        if let Some(click_pos) = click_pos {
                            if rect1.contains(click_pos) {
                                if response.dragged_by(egui::PointerButton::Primary) {
                                    grid.cell_mut(i, j).fill();
                                } else if response.dragged_by(egui::PointerButton::Secondary) {
                                    grid.cell_mut(i, j).make_wall();
                                } else if response.dragged_by(egui::PointerButton::Middle) {
                                    grid.cell_mut(i, j).clear_wall();
                                }
                            }
                        }

                        if grid.cell(i, j).is_wall() {
                            shapes.push(Shape::rect_filled(rect1, 0.0, Color32::GRAY))
                        } else {
                            shapes.push(Shape::rect_filled(rect2, 0.0, Color32::LIGHT_BLUE));
                            shapes.push(Shape::rect_filled(rect3, 0.0, Color32::BLUE));
                        }

                        shapes.push(Shape::line_segment(
                            [
                                to_screen * pos,
                                to_screen * pos + velocity.normalized() * 5.0,
                            ],
                            Stroke::new(1.0, Color32::BLACK),
                        ));

                        shapes.push(Shape::rect_stroke(
                            rect1,
                            0.0,
                            Stroke::new(1.0, Color32::WHITE),
                        ));
                    }
                }

                painter.extend(shapes);

                if let Some(last_update) = last_update {
                    let mut delta = (ctx.input().time - *last_update).clamp(0.0, 0.5);

                    while delta > 0.0 {
                        delta -= 1.0 / 30.0;
                        grid.update(*enable_gravity, *enable_inertia);
                    }
                }
                *last_update = Some(ctx.input().time);

                if *show_total_water {
                    dbg!(grid.total_water());
                }
            });
        });
    }
}
