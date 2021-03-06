use super::{DeathTrigger, Health, Map, Position, RunState};
use specs::prelude::*;

pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RunState>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, DeathTrigger>,
        ReadStorage<'a, Health>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player, mut map, mut run_state, positions, death_triggers, healths) = data;
        let mut dead = Vec::new();

        for (ent, pos, health, effect) in
            (&entities, &positions, &healths, (&death_triggers).maybe()).join()
        {
            if health.current <= 0 {
                if let Some(effect) = effect {
                    crate::add_event(
                        &effect.event,
                        None,
                        None,
                        &effect.range,
                        pos.as_point(),
                        true,
                    );
                }

                if ent != *player {
                    dead.push(ent);
                    map.untrack_creature(rltk::Point::new(pos.x, pos.y));
                } else {
                    *run_state = RunState::Dead;
                }
            }
        }

        for victim in dead {
            entities
                .delete(victim)
                .expect("Failed to remove dead entity");
        }
    }
}
