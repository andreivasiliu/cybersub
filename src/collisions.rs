use crate::{app::Navigation, rocks::RockGrid, water::WaterGrid};

pub(crate) fn update_collisions(
    water_grid: &mut WaterGrid,
    rock_grid: &mut RockGrid,
    navigation: &Navigation,
) {
    let world_size = rock_grid.size();
    let sub_size = water_grid.size();

    let mut sub_collisions = Vec::new();

    for y in 0..sub_size.1 {
        for x in 0..sub_size.0 {
            water_grid.cell_mut(x, y).set_collided(false);
        }
    }

    for y in 0..world_size.1 {
        for x in 0..world_size.0 {
            rock_grid.cell_mut(x, y).set_collided(false);
        }
    }

    for &(sub_x, sub_y) in water_grid.edges() {
        let (rock_x, rock_y) = (
            ((navigation.position.0 / 16 + sub_x as i32) / 16).clamp(0, world_size.0 as i32 - 1),
            ((navigation.position.1 / 16 + sub_y as i32) / 16).clamp(0, world_size.1 as i32 - 1),
        );

        let rock_cell = rock_grid.cell_mut(rock_x as usize, rock_y as usize);

        if rock_cell.is_wall() {
            // The point inside the rock cell where the submarine cell is
            let inner_x = (navigation.position.0 / 16 + sub_x as i32) % 16;
            let inner_y = (navigation.position.1 / 16 + sub_y as i32) % 16;

            let collided = match rock_cell.rock_type() {
                crate::rocks::RockType::Empty => unreachable!(),
                crate::rocks::RockType::WallFilled => true,
                crate::rocks::RockType::WallLowerLeft => inner_x < inner_y,
                crate::rocks::RockType::WallLowerRight => (15 - inner_x) < inner_y,
                crate::rocks::RockType::WallUpperLeft => (15 - inner_x) > inner_y,
                crate::rocks::RockType::WallUpperRight => inner_x > inner_y,
            };

            if collided {
                rock_cell.set_collided(true);
                sub_collisions.push((sub_x, sub_y));
            }
        }
    }

    for &(sub_x, sub_y) in &sub_collisions {
        let sub_cell = water_grid.cell_mut(sub_x, sub_y);

        sub_cell.set_collided(true);
    }
}
