use super::{CanActFlag, Position, RunState, Schedulable};
use specs::prelude::*;

pub struct TurnSystem;

impl<'a> System<'a> for TurnSystem {
    type SystemData = (
        WriteExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, CanActFlag>,
        WriteStorage<'a, Schedulable>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut game_state, entities, mut can_act, mut schedulables, pos, player) = data;
        assert!(*game_state == RunState::Running);

        if can_act.get(*player).is_some() {
            *game_state = RunState::AwaitingInput;
            return;
        }

        for (ent, sched, _pos) in (&entities, &mut schedulables, &pos).join() {
            sched.current -= sched.delta;
            if sched.current > 0 {
                continue;
            }

            sched.current += sched.base;
            can_act
                .insert(ent, CanActFlag { is_reaction: false })
                .expect("Failed to insert CanActFlag");
        }
    }
}
