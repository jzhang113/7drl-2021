use super::AttackIntent;
use specs::prelude::*;

pub struct AttackSystem;

impl<'a> System<'a> for AttackSystem {
    type SystemData = (Entities<'a>, WriteStorage<'a, AttackIntent>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut attacks) = data;

        for (_, intent) in (&entities, &attacks).join() {
            crate::add_damage_event(intent, true);
        }

        attacks.clear();
    }
}
