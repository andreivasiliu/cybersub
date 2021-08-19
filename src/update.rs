use crate::{
    app::{GameSettings, GameState},
    collisions::{update_rock_collisions, update_submarine_collisions},
    objects::update_objects,
    resources::{MutableResources, MutableSubResources},
    sonar::update_sonar,
};

pub(crate) fn update_game(
    game_state: &mut GameState,
    game_settings: &mut GameSettings,
    mutable_resources: &mut MutableResources,
    mutable_sub_resources: &mut [MutableSubResources],
) {
    let update_settings = &game_settings.update_settings;

    mutable_resources.collisions.clear();

    for (sub_index, submarine) in game_state.submarines.iter_mut().enumerate() {
        let mutable_sub_resources = mutable_sub_resources
            .get_mut(sub_index)
            .expect("All submarines should have a MutableSubResources instance");

        if update_settings.update_position {
            let navigation = &mut submarine.navigation;

            // Compute weight based on number of walls
            let weight = submarine.water_grid.total_walls() as i32;

            // Compute buoyancy; the numbers are just random stuff that seems to
            // somewhat work for both the Dugong and the Bunyip
            let mut buoyancy = 0;
            buoyancy -= weight * 16;
            buoyancy += submarine.water_grid.total_inside() as i32 * 13;
            buoyancy -= submarine.water_grid.total_water() as i32 * 16 / 1024;

            // Massive submarines are harder to move
            let mass = (weight * weight / 1500 / 1500).max(1);

            let y_acceleration = (buoyancy * weight) / 1024 / 100;
            navigation.acceleration.1 = -y_acceleration / 8 / mass;

            navigation.speed.0 =
                (navigation.speed.0 + navigation.acceleration.0).clamp(-2048, 2048);
            navigation.speed.1 =
                (navigation.speed.1 + navigation.acceleration.1).clamp(-2048, 2048);

            navigation.position.0 += navigation.speed.0 / 256;
            navigation.position.1 += navigation.speed.1 / 256;
        }

        if update_settings.update_water {
            submarine
                .water_grid
                .update(game_settings.enable_gravity, game_settings.enable_inertia);
        }
        if update_settings.update_wires {
            for _ in 0..3 {
                submarine
                    .wire_grid
                    .update(&mut mutable_sub_resources.signals_updated);
            }
        }
        if update_settings.update_objects {
            update_objects(submarine, mutable_sub_resources);
        }
        if update_settings.update_sonar {
            update_sonar(
                &mut submarine.sonar,
                &submarine.navigation,
                submarine.water_grid.size(),
                &game_state.rock_grid,
                mutable_sub_resources,
            );
        }

        if update_settings.update_collision {
            mutable_sub_resources.collisions.clear();
            update_rock_collisions(
                &submarine.water_grid,
                &game_state.rock_grid,
                &submarine.navigation,
                mutable_resources,
                mutable_sub_resources,
            );
        }
    }

    if update_settings.update_collision {
        for (sub1_index, submarine1) in game_state.submarines.iter().enumerate() {
            for (sub2_index, submarine2) in game_state.submarines.iter().enumerate() {
                if sub1_index == sub2_index {
                    continue;
                }

                let mutable_resources = mutable_sub_resources
                    .get_mut(sub1_index)
                    .expect("All submarines should have a MutableSubResources instance");

                update_submarine_collisions(
                    &submarine1.water_grid,
                    &submarine2.water_grid,
                    &submarine1.navigation,
                    &submarine2.navigation,
                    mutable_resources,
                );
            }
        }
    }

    let submarine_camera = game_state
        .submarines
        .get(game_settings.current_submarine)
        .map(|submarine| {
            (
                submarine.navigation.position.0,
                submarine.navigation.position.1,
            )
        });

    game_settings.camera.current_submarine = submarine_camera;
}
