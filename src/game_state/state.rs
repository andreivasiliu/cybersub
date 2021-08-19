use super::{objects::Object, rocks::RockGrid, sonar::Sonar, water::WaterGrid, wires::WireGrid};

#[derive(Clone, PartialEq, Eq)]
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

pub(crate) struct GameState {
    pub update_settings: UpdateSettings,
    pub last_update: Option<f64>,
    pub rock_grid: RockGrid,
    pub submarines: Vec<SubmarineState>,
    pub collisions: Vec<(usize, usize)>,
}

pub(crate) struct SubmarineState {
    pub water_grid: WaterGrid,
    pub wire_grid: WireGrid,
    pub objects: Vec<Object>,
    pub sonar: Sonar,
    pub navigation: Navigation,
    pub collisions: Vec<(usize, usize)>,
}

#[derive(Default)]
pub(crate) struct Navigation {
    pub target: (i32, i32),
    pub position: (i32, i32),
    pub speed: (i32, i32),
    pub acceleration: (i32, i32),
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
            last_update: None,
            rock_grid: RockGrid::new(0, 0),
            submarines: Vec::new(),
            collisions: Vec::new(),
        }
    }
}
