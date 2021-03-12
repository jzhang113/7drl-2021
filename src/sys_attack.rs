use super::AttackIntent;
use specs::prelude::*;

pub struct AttackSystem;

impl<'a> System<'a> for AttackSystem {
    type SystemData = (Entities<'a>, WriteStorage<'a, AttackIntent>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut attacks) = data;

<<<<<<< HEAD
        for (_, intent) in (&entities, &attacks).join() {
            crate::add_damage_event(intent, true);
=======
        for (ent, attack) in (&entities, &attacks).join() {
            crate::add_event(
                &crate::EventType::Damage {
                    source_name: attack.name.clone(),
                    amount: attack.damage,
                },
                Some(ent),
                &attack.range,
                attack.loc,
                true,
            )
>>>>>>> 8a3507a... supply source for attacks
        }

        attacks.clear();
    }
}
