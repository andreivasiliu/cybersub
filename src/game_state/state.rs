use serde::{Deserialize, Serialize};

use super::{
    objects::Object,
    rocks::RockGrid,
    sonar::Sonar,
    water::{CellTemplate, WaterGrid},
    wires::{WireGrid, WirePoints},
};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct UpdateSettings {
    pub update_water: bool,
    pub enable_gravity: bool,
    pub enable_inertia: bool,
    pub update_wires: bool,
    pub update_sonar: bool,
    pub update_objects: bool,
    pub update_position: bool,
    pub update_collision: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GameState {
    pub update_settings: UpdateSettings,
    pub rock_grid: RockGrid,
    pub submarines: Vec<SubmarineState>,
    pub collisions: Vec<(usize, usize)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct SubmarineState {
    pub background_pixels: Vec<u8>,
    pub water_grid: WaterGrid,
    pub wire_grid: WireGrid,
    pub objects: Vec<Object>,
    pub sonar: Sonar,
    pub navigation: Navigation,
    pub collisions: Vec<(usize, usize)>,
    pub docking_points: Vec<DockingPoint>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub(crate) struct Navigation {
    pub target: (i32, i32),
    pub position: (i32, i32),
    pub speed: (i32, i32),
    pub docking_override: (i32, i32),
    pub acceleration: (i32, i32),
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DockingPoint {
    pub connection_point: (i32, i32),
    pub connector_object_id: usize,
    pub connected_to: Option<(usize, usize)>,
    pub in_proximity_to: Option<(i32, i32)>,
    pub speed_offset: (i32, i32),
    pub direction: DockingDirection,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(crate) enum DockingDirection {
    Top,
    Bottom,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SubmarineTemplate {
    pub size: (usize, usize),
    pub water_cells: Vec<CellTemplate>,
    pub background_pixels: Vec<u8>,
    pub objects: Vec<Object>,
    pub wire_points: Vec<WirePoints>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        UpdateSettings {
            update_water: !cfg!(debug_assertions), // Very expensive in debug mode
            enable_gravity: true,
            enable_inertia: true,
            update_wires: true,
            update_sonar: true,
            update_objects: true,
            update_position: true,
            update_collision: true,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            update_settings: UpdateSettings::default(),
            rock_grid: RockGrid::new(0, 0),
            submarines: Vec::new(),
            collisions: Vec::new(),
        }
    }
}
