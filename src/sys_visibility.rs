use super::{Map, Player, Position, Viewable, Viewshed};
use rltk::{Algorithm2D, Point};
use specs::prelude::*;

pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Viewable>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewsheds, mut viewables, pos, player) = data;

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
                    }
                }
            }
        }

        // TODO: we can probably set up the list index here instead of in gui
        for mut view in (&mut viewables).join() {
            view.list_index = None;
        }
    }
}
