use super::{MoveIntent, Position, Viewshed};
use specs::prelude::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, MoveIntent>,
        WriteStorage<'a, Viewshed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut movements, mut viewsheds) = data;

        for (_, pos, movement, viewshed) in (
            &entities,
            &mut positions,
            &movements,
            (&mut viewsheds).maybe(),
        )
            .join()
        {
            let new_pos = movement.loc;
            pos.x = new_pos.x;
            pos.y = new_pos.y;

            if let Some(viewshed) = viewshed {
                viewshed.dirty = true;
            }
        }

        movements.clear();
    }
}
