use super::{Map, MoveIntent, Position, Viewshed};
use specs::prelude::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, MoveIntent>,
        WriteStorage<'a, Viewshed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut map, mut positions, mut movements, mut viewsheds) = data;

        for (_, pos, movement, viewshed) in (
            &entities,
            &mut positions,
            &movements,
            (&mut viewsheds).maybe(),
        )
            .join()
        {
            let new_pos = movement.loc;
            let prev_index = map.get_index(pos.x, pos.y);
            let new_index = map.get_index(new_pos.x, new_pos.y);

            // check if the tile is blocked, since it may have changed
            if !map.blocked_tiles[new_index] {
                map.blocked_tiles[prev_index] = false;
                map.blocked_tiles[new_index] = true;

                pos.x = new_pos.x;
                pos.y = new_pos.y;

                if let Some(viewshed) = viewshed {
                    viewshed.dirty = true;
                }
            }
        }

        movements.clear();
    }
}
