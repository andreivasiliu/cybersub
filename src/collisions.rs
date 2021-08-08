use crate::{
    app::Navigation,
    resources::{MutableResources, MutableSubResources},
    rocks::RockGrid,
    water::WaterGrid,
};

pub(crate) fn update_collisions(
    water_grid: &mut WaterGrid,
    rock_grid: &mut RockGrid,
    navigation: &Navigation,
    mutable_resources: &mut MutableResources,
    mutable_sub_resources: &mut MutableSubResources,
) {
    let world_size = rock_grid.size();

    for &(sub_x, sub_y) in water_grid.edges() {
        let (rock_x, rock_y) = (
            ((navigation.position.0 / 16 + sub_x as i32) / 16).clamp(0, world_size.0 as i32 - 1),
            ((navigation.position.1 / 16 + sub_y as i32) / 16).clamp(0, world_size.1 as i32 - 1),
        );
        let (rock_x, rock_y) = (rock_x as usize, rock_y as usize);

        let rock_cell = rock_grid.cell_mut(rock_x, rock_y);

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
                mutable_resources.collisions.push((rock_x, rock_y));
                mutable_sub_resources.collisions.push((sub_x, sub_y));
            }
        }
    }
}
