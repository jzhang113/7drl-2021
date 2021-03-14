use super::{DeathTrigger, Health, Position, RunState};
use specs::prelude::*;

pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, RunState>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, DeathTrigger>,
        ReadStorage<'a, Health>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player, mut run_state, positions, death_triggers, healths) = data;
        let mut dead = Vec::new();

        for (ent, pos, health, effect) in
            (&entities, &positions, &healths, (&death_triggers).maybe()).join()
        {
            if health.current <= 0 {
                if let Some(effect) = effect {
                    crate::add_event(&effect.event, None, &effect.range, pos.as_point(), true);
                }

                if ent != *player {
                    dead.push(ent);
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
