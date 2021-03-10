use super::{BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexSystem;

impl<'a> System<'a> for MapIndexSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers) = data;

        map.set_blocked_tiles();
        for (pos, _blocked) in (&positions, &blockers).join() {
            let index = map.get_index(pos.x, pos.y);
            map.blocked_tiles[index] = true;
        }
    }
}
