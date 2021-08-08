use crate::{
    app::Navigation,
    resources::{MutableResources, MutableSubResources},
    rocks::RockGrid,
    water::WaterGrid,
};

pub(crate) fn update_rock_collisions(
    water_grid: &WaterGrid,
    rock_grid: &RockGrid,
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

        let rock_cell = rock_grid.cell(rock_x, rock_y);

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

pub(crate) fn update_submarine_collisions(
    grid1: &WaterGrid,
    grid2: &WaterGrid,
    navigation1: &Navigation,
    navigation2: &Navigation,
    mutable_resources1: &mut MutableSubResources,
) {
    // TODO: Do a general "are the grid even overlapping?" check first; although
    // right now this is barely taking any time at all, despite being O(n^2).

    for &(sub1_x, sub1_y) in grid1.edges() {
        let sub2_x = sub1_x as i32 + (navigation1.position.0 - navigation2.position.0) / 16;
        let sub2_y = sub1_y as i32 + (navigation1.position.1 - navigation2.position.1) / 16;

        let (width2, height2) = grid2.size();

        if sub2_x < 0 || sub2_y < 0 || sub2_x >= width2 as i32 || sub2_y >= height2 as i32 {
            continue;
        }

        let cell2 = grid2.cell(sub2_x as usize, sub2_y as usize);

        if !cell2.is_sea() {
            mutable_resources1
                .collisions
                .push((sub1_x as usize, sub1_y as usize));
        }
    }
}