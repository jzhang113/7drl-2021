use super::{Map, Player, Position, Viewable, ViewableIndex, Viewshed};
use rltk::{Algorithm2D, Point};
use specs::prelude::*;

pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, ViewableIndex>,
        WriteStorage<'a, Viewable>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewsheds, mut view_indexes, mut viewables, pos, player) = data;
        let mut player_seen = Vec::new();

        for (ent, viewshed, pos) in (&entities, &mut viewsheds, &pos).join() {
            if !viewshed.dirty {
                continue;
            }

            viewshed.visible.clear();
            viewshed.visible = rltk::field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed
                .visible
                .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);
            viewshed.dirty = false;

            match player.get(ent) {
                None => {}
                Some(_) => {
                    for seen in map.visible_tiles.iter_mut() {
                        *seen = false
                    }

                    for pos in viewshed.visible.iter() {
                        let index = map.point2d_to_index(*pos);
                        map.known_tiles[index] = true;
                        map.visible_tiles[index] = true;

                        if let Some(seen_ent) = map.creature_map.get(&index) {
                            player_seen.push(*seen_ent);
                        }
                    }
                }
            }
        }

        // only update the view index for the player's viewshed
        if player_seen.len() > 0 {
            let mut index = 0;
            for (ent, mut v_index) in (&entities, &mut view_indexes).join() {
                if player_seen.contains(&ent) {
                    v_index.list_index = Some(index);
                    index += 1;
                } else {
                    v_index.list_index = None;
                }
            }
        }
    }
}
