//! Main idea:
//! * Generate an empty submarine-sized render target
//! * For each light:
//!   * Generate/load a square texture for a light
//!   * Generate polygon or triangle-fan based on visibility from origin
//!     * Can be cached if nothing changed recently
//!   * Draw polygon/triangle-fan with light texture on render target
//! * End result is a light map
//! * Draw game as usual
//! * Draw inverse of light map onto game with black or fog texture
//!   * Can use a fuzzy shader for soft shadow edges
//!   * This light map can be cached if nothing changed on the grid
//!
//! Workflow for generating polygon based on visibility from origin:
//! * Generate all edges on the grid
//!   * Can be cached if nothing changed recently
//! * Find all edges in vicinity, add border edges
//!   * Can be cached if nothing changed recently
//! * Determine whether each point starts or ends the edge when rotating clockwise
//! * Calculate radians/distance for each point
//! * Create empty edge set
//! * Raycast all edges with a ray pointing straight up, add all to set
//! * Save closest edge's (0, y) point as last_point
//! * Sort all edge points by radians
//!   * To prevent tiny gaps, order edge starts before edge ends
//! * Iterate clockwise over points
//! * If the next point starts an edge:
//!   * Add the edge to the set
//!   * If it is now the closest in the set:
//!     * Generate a point for the next closest_edge_point in the set
//!     * Commit triangle based on: origin, last_point, closest_edge_point
//!     * Save edge-starting-point as last_point
//! * If the next point ends an edge:
//!   * Remove from edge set
//!   * If it was the closest one in the set:
//!     * Generate a point for the next closest_edge_point in the set
//!     * Commit triangle based on: origin, last_point, removed_edge_point
//!     * Save closest_edge_point as last_point
//!
//! Workflow for generating a point for the closest edge in the set:
//! * Normal raycast (aka line intersection equation) maybe?
//! * Some sort of start/end radians interpolation?

use std::{f32::consts::PI, fmt::Display};

use macroquad::prelude::{vec2, Rect, Vec2};

use crate::game_state::water::WaterGrid;

struct EdgeGrid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

#[derive(Default, Clone, Copy)]
struct Cell {
    edges: [Option<usize>; 4],
}

#[derive(Clone, Copy)]
pub(crate) struct Edge {
    pub edge_type: Direction,
    pub start_cell: (usize, usize),
    pub end_cell: (usize, usize),
    line: (Vec2, Vec2),
    start_point: EdgePoint,
    end_point: EdgePoint,
}

#[derive(Default, Clone, Copy)]
struct EdgePoint {
    starts_edge: bool,
    point: Vec2,
    radians: f32,
    clock_radians: f32,
    edge_index: usize,
}

#[derive(Clone, Copy)]
#[repr(usize)]
pub(crate) enum Direction {
    Top,
    Right,
    Bottom,
    Left,
}

pub(crate) struct Triangle(pub Vec2, pub Vec2, pub Vec2);

/// Allow ordering floats by panicking on NaNs
#[derive(PartialOrd, PartialEq)]
struct R32(f32);

impl Eq for R32 {}

impl Ord for R32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("There should be no NaNs")
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Top => "top",
            Direction::Right => "right",
            Direction::Bottom => "bottom",
            Direction::Left => "left",
        }
        .fmt(f)
    }
}

impl std::fmt::Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Edge {{ {}, {} ({}), {} ({}) }}",
            self.edge_type,
            self.start_point.point,
            self.start_point.radians,
            self.end_point.point,
            self.end_point.radians
        )
    }
}

impl Edge {
    pub(crate) fn line(&self) -> (Vec2, Vec2) {
        self.line
    }

    pub(crate) fn make_line(&self) -> (Vec2, Vec2) {
        let (s_x, s_y) = match self.edge_type {
            Direction::Top => (0, 0),
            Direction::Right => (1, 0),
            Direction::Bottom => (0, 1),
            Direction::Left => (0, 0),
        };

        let (e_x, e_y) = match self.edge_type {
            Direction::Top => (1, 0),
            Direction::Right => (1, 1),
            Direction::Bottom => (1, 1),
            Direction::Left => (0, 1),
        };

        let start = (self.start_cell.0 + s_x, self.start_cell.1 + s_y);
        let end = (self.end_cell.0 + e_x, self.end_cell.1 + e_y);

        let start = vec2(start.0 as f32, start.1 as f32);
        let end = vec2(end.0 as f32, end.1 as f32);

        (start, end)
    }
}

impl EdgeGrid {
    fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::new();
        cells.resize(width * height, Cell::default());

        EdgeGrid {
            cells,
            width,
            height,
        }
    }

    fn cell(&self, x: usize, y: usize) -> &Cell {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        &self.cells[y * self.width + x]
    }

    fn cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        &mut self.cells[y * self.width + x]
    }
}

impl Cell {
    fn edge(&self, edge: Direction) -> Option<usize> {
        self.edges[edge as usize]
    }

    fn edge_mut(&mut self, edge: Direction) -> &mut Option<usize> {
        &mut self.edges[edge as usize]
    }
}

fn has_edge(water_grid: &WaterGrid, x: usize, y: usize, edge: Direction) -> bool {
    if !water_grid.cell(x, y).is_wall() {
        return false;
    }

    let neighbour = match neighbour(water_grid, x, y, edge) {
        Some(neighbour) => neighbour,
        None => return false,
    };

    !water_grid.cell(neighbour.0, neighbour.1).is_wall()
}

fn continues_edge(
    water_grid: &WaterGrid,
    edge_grid: &EdgeGrid,
    x: usize,
    y: usize,
    edge: Direction,
) -> Option<usize> {
    let continues_from = match edge {
        Direction::Top => Direction::Left,
        Direction::Bottom => Direction::Left,
        Direction::Right => Direction::Top,
        Direction::Left => Direction::Top,
    };

    let neighbour = match neighbour(water_grid, x, y, continues_from) {
        Some(neighbour) => neighbour,
        None => return None,
    };

    let (x, y) = neighbour;

    edge_grid.cell(x, y).edge(edge)
}

fn neighbour(
    water_grid: &WaterGrid,
    x: usize,
    y: usize,
    edge: Direction,
) -> Option<(usize, usize)> {
    let (width, height) = water_grid.size();

    let neighbour = match (edge, x, y) {
        (Direction::Top, _x, y) if y == 0 => return None,
        (Direction::Bottom, _x, y) if y >= height - 1 => return None,
        (Direction::Left, x, _y) if x == 0 => return None,
        (Direction::Right, x, _y) if x >= width - 1 => return None,
        (Direction::Top, x, y) => (x, y - 1),
        (Direction::Bottom, x, y) => (x, y + 1),
        (Direction::Left, x, y) => (x - 1, y),
        (Direction::Right, x, y) => (x + 1, y),
    };

    Some(neighbour)
}

pub(crate) fn find_shadow_edges(water_grid: &WaterGrid) -> Vec<Edge> {
    let (width, height) = water_grid.size();
    let mut edge_grid = EdgeGrid::new(width, height);
    let mut edges: Vec<Edge> = Vec::new();

    let directions = [
        Direction::Top,
        Direction::Right,
        Direction::Bottom,
        Direction::Left,
    ];

    for y in 0..height {
        for x in 0..width {
            for edge_type in directions {
                if has_edge(water_grid, x, y, edge_type) {
                    let new_edge = match continues_edge(water_grid, &edge_grid, x, y, edge_type) {
                        Some(edge) => {
                            edges[edge].end_cell = (x, y);
                            edge
                        }
                        None => {
                            let edge = Edge {
                                edge_type,
                                start_cell: (x, y),
                                end_cell: (x, y),
                                line: (Vec2::ZERO, Vec2::ZERO),
                                start_point: EdgePoint::default(),
                                end_point: EdgePoint::default(),
                            };
                            edges.push(edge);

                            edges.len() - 1
                        }
                    };

                    *edge_grid.cell_mut(x, y).edge_mut(edge_type) = Some(new_edge);
                }
            }
        }
    }

    for edge in &mut edges {
        edge.line = edge.make_line();
    }

    edges
}

pub(crate) fn filter_edges_by_region(edges: &[Edge], cursor: Vec2, range: f32) -> Vec<Edge> {
    let region = Rect::new(cursor.x - range, cursor.y - range, range * 2.0, range * 2.0);
    let mut edges_in_region = Vec::new();

    let is_in_region = |edge: &Edge| {
        let (p1, p2) = edge.line();

        // Check if one of the edge points are in the region
        if region.contains(p1) || region.contains(p2) {
            return true;
        }

        // Check if a line cross through the region
        let cross_x =
            (p1.x..=p2.x).contains(&cursor.x) && (region.top()..=region.bottom()).contains(&p1.y);
        let cross_y =
            (p1.y..=p2.y).contains(&cursor.y) && (region.left()..=region.right()).contains(&p1.x);

        cross_x || cross_y
    };

    for edge in edges {
        if is_in_region(&edge) {
            edges_in_region.push(*edge);
        }
    }

    edges_in_region
}

pub(crate) fn add_border_edges(edges: &mut Vec<Edge>, cursor: Vec2, range: f32) {
    // Add border edges
    let border_lines = [
        (Direction::Bottom, -1.0, -1.0, 1.0, -1.0),
        (Direction::Left, 1.0, -1.0, 1.0, 1.0),
        (Direction::Top, -1.0, 1.0, 1.0, 1.0),
        (Direction::Right, -1.0, -1.0, -1.0, 1.0),
    ];

    for (dir, s_dir_x, s_dir_y, e_dir_x, e_dir_y) in border_lines {
        let start = vec2(cursor.x + range * s_dir_x, cursor.y + range * s_dir_y);
        let end = vec2(cursor.x + range * e_dir_x, cursor.y + range * e_dir_y);

        edges.push(Edge {
            edge_type: dir,
            start_cell: (0, 0),
            end_cell: (0, 0),
            line: (start, end),
            start_point: EdgePoint::default(),
            end_point: EdgePoint::default(),
        });
    }
}

pub(crate) fn filter_edges_by_direction(mut edges: Vec<Edge>, cursor: Vec2) -> Vec<Edge> {
    edges.retain(|edge| {
        let (p1, _p2) = edge.line;

        match edge.edge_type {
            Direction::Top => cursor.y < p1.y,
            Direction::Right => cursor.x > p1.x,
            Direction::Bottom => cursor.y > p1.y,
            Direction::Left => cursor.x < p1.x,
        }
    });

    edges
}

fn generate_edge_points(edges: &mut Vec<Edge>, cursor: Vec2, range: f32) -> Vec<EdgePoint> {
    let mut points = Vec::new();

    // Clamping range
    let r_left = cursor.x - range;
    let r_right = cursor.x + range;
    let r_top = cursor.y - range;
    let r_bottom = cursor.y + range;

    for (edge_index, edge) in edges.iter_mut().enumerate() {
        let reversed = match edge.edge_type {
            Direction::Top => true,
            Direction::Right => true,
            Direction::Bottom => false,
            Direction::Left => false,
        };

        let (start, end) = edge.line;

        let start = vec2(
            start.x.clamp(r_left, r_right),
            start.y.clamp(r_top, r_bottom),
        );
        let end = vec2(end.x.clamp(r_left, r_right), end.y.clamp(r_top, r_bottom));

        let line = (start, end);

        if line.0 == cursor || line.1 == cursor {
            // One point would have no angle compared to the cursor
            continue;
        }

        // Angle compared to up from -PI to PI, with 0 being up
        let radians = -(line.0 - cursor).angle_between(vec2(0.0, -1.0));

        // And also from 0 to 2*PI, with 0 still being up
        let clock_radians = if radians < 0.0 {
            radians + 2.0 * PI
        } else {
            radians
        };

        edge.start_point = EdgePoint {
            starts_edge: !reversed,
            point: line.0,
            radians,
            clock_radians,
            edge_index,
        };

        let radians = -(line.1 - cursor).angle_between(vec2(0.0, -1.0));
        let clock_radians = if radians < 0.0 {
            radians + 2.0 * PI
        } else {
            radians
        };

        edge.end_point = EdgePoint {
            starts_edge: reversed,
            point: line.1,
            radians,
            clock_radians,
            edge_index,
        };

        if reversed {
            std::mem::swap(&mut edge.start_point, &mut edge.end_point);
        }

        points.push(edge.start_point);
        points.push(edge.end_point);
    }

    // Sort all edge points clockwise.
    // If two points are in the same place, sort the one that starts an edge
    // first; this is so there is always an edge added to the current edge set
    // when scanning clockwise. Note: false < true.
    points.sort_unstable_by_key(|point| (R32(point.clock_radians), !point.starts_edge));

    points
}

pub(crate) fn find_shadow_triangles(
    mut edges: Vec<Edge>,
    cursor: Vec2,
    range: f32,
) -> (Vec<Triangle>, Vec<Vec2>) {
    let points = generate_edge_points(&mut edges, cursor, range);

    // Remove `mut`.
    let edges = edges;

    let mut current_edges = Vec::new();

    // eprintln!("Cursor: {}", cursor);

    for (edge_index, edge) in edges.iter().enumerate() {
        // Radians go from -PI to PI, with 0 being straight up.
        // If these points are on two sides of 0, that means the line is
        // straight upwards (starting point on the left and ending point on
        // the right).
        if edge.start_point.radians < 0.0 && edge.end_point.radians >= 0.0 {
            // eprintln!("Starting edge: {:?}", edge);
            current_edges.push(edge_index);
        }
    }

    // Find the starting point: whatever's closest pointing straight up.
    current_edges.sort_unstable_by_key(|index: &usize| R32(-edges[*index].start_point.point.y));
    let starting_edge = *current_edges
        .first()
        .expect("Should have at least border edge lines");

    let starting_point = vec2(cursor.x, edges[starting_edge].start_point.point.y);
    let mut last_point = starting_point;

    let mut distances = Vec::new();
    distances.resize(edges.len(), 0.0);

    let mut triangles = Vec::new();

    // eprintln!(
    //     "Starting with: {}",
    //     current_edges
    //         .iter()
    //         .map(|i| format!("{} ({})", i, edges[*i].edge_type))
    //         .collect::<Vec<_>>()
    //         .join(", ")
    // );

    let mut three_points = Vec::new();

    // eprintln!("Points:");
    // for point in &points {
    //     eprintln!(" * {} {}: {:?}", point.point, if point.starts_edge { "starts" } else { "ends" }, edges[point.edge_index]);
    // }

    for point in points {
        three_points.push(point.point);

        let point_distance = point.point - cursor;
        let ray = point_distance.normalize();

        if point.starts_edge {
            current_edges.push(point.edge_index);
            // eprintln!("Added {} ({}): {:?}", point.edge_index, current_edges.len(), edges[point.edge_index]);
        } else {
            current_edges.retain(|edge_index| point.edge_index != *edge_index);
            // eprintln!("Removed {} ({}): {:?}", point.edge_index, current_edges.len(), &edges[point.edge_index]);
        };

        // eprintln!(
        //     "Edges: {}",
        //     current_edges
        //         .iter()
        //         .map(|i| format!("{} ({})", i, edges[*i].edge_type))
        //         .collect::<Vec<_>>()
        //         .join(", ")
        // );

        // Calculate the distance of other edges' intersections on this ray
        for &edge_index in &current_edges {
            let edge = edges[edge_index];
            let line = (edge.start_point.point, edge.end_point.point);
            distances[edge_index] = intersection_distance((cursor, ray), line);
        }

        if point.starts_edge {
            let point_distance = point_distance.length();
            let closest_distance = current_edges
                .iter()
                .map(|edge_index| distances[*edge_index])
                .min_by_key(|distance| R32(*distance))
                .expect("Should have at least border edge lines");

            // eprintln!(
            //     "[{}] Closest on add: {}/{}",
            //     point.edge_index,
            //     closest_edge,
            //     edges.len()
            // );

            if point_distance <= closest_distance + 0.1 {
                let next_closest = *current_edges
                    .iter()
                    .filter(|edge_index| **edge_index != point.edge_index)
                    .min_by_key(|edge_index| R32(distances[**edge_index]))
                    .expect("Pushed one in this iteration");

                let distance = distances[next_closest];

                let next_closest_point = cursor + ray * distance;

                triangles.push(Triangle(cursor, last_point, next_closest_point));

                // eprintln!("Added triangle on add.");

                last_point = point.point;
            }
        } else {
            let point_distance = point_distance.length();

            // eprintln!("Added triangle on remove.");

            let next_closest_distance = current_edges
                .iter()
                .map(|edge_index| distances[*edge_index])
                .min_by_key(|distance| R32(*distance))
                .expect("Should have at least border edge lines");

            // eprintln!(
            //     "[{}] Closest on remove: {}/{}",
            //     point.edge_index, next_closest_distance, point_distance
            // );

            let was_closest = point_distance <= next_closest_distance + 0.1;

            if was_closest {
                let next_closest_point = cursor + ray * next_closest_distance;

                triangles.push(Triangle(cursor, last_point, point.point));

                last_point = next_closest_point;
            }
        }
    }

    triangles.push(Triangle(cursor, last_point, starting_point));

    // dbg!(triangles.len());

    (triangles, three_points)
}

/// Find where a ray and line intersect
fn intersection_distance(ray: (Vec2, Vec2), line: (Vec2, Vec2)) -> f32 {
    let (point, direction) = ray;
    let (line_p1, line_p2) = line;

    let line_direction = line_p2 - line_p1;

    let projection = direction.perp_dot(line_direction);

    let projection = if projection != 0.0 {
        projection
    } else {
        // The lines are parallel; just make it be really far into the
        // distance, it's better than dealing with a NaN.
        0.001
    };

    ((line_p1 - point).perp_dot(line_direction)) / projection
}
