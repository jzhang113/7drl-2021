use super::{Heal, Health, Map, Position, RunState, Schedulable, SkillChoice};
use specs::prelude::*;

pub struct PickupSystem;

impl<'a> System<'a> for PickupSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RunState>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Schedulable>,
        ReadStorage<'a, Heal>,
        ReadStorage<'a, SkillChoice>,
        WriteStorage<'a, Health>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player,
            mut map,
            mut run_state,
            positions,
            scheds,
            heals,
            skills,
            mut healths,
        ) = data;
        let mut consumed = Vec::new();

        for (ent, pos, mut health, _) in (&entities, &positions, &mut healths, &scheds).join() {
            let point = rltk::Point::new(pos.x, pos.y);

            match map.untrack_item(point) {
                None => {}
                Some(item_ent) => {
                    if let Some(healing) = heals.get(item_ent) {
                        health.current += healing.amount as i32;
                        health.current = std::cmp::min(health.current, health.max);
                        consumed.push(item_ent);
                    } else if let Some(skill_book) = skills.get(item_ent) {
                        if *player == ent {
                            let mut choice_ary = [None; 4];
                            for (i, att_type) in skill_book.choices.iter().enumerate().take(4) {
                                choice_ary[i] = Some(*att_type);
                            }

                            *run_state = RunState::ChooseReward {
                                choices: choice_ary,
                            };
                        }

                        consumed.push(item_ent);
                    }
                }
            }
        }

        for item in consumed {
            entities
                .delete(item)
                .expect("Failed to remove consumed item");
        }
    }
}
