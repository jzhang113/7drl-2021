use super::{deck::Deck, AttackIntent, Position};
use crate::move_type;
use specs::prelude::*;

pub struct AttackSystem;

impl<'a> System<'a> for AttackSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Deck>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, AttackIntent>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player, mut deck, positions, mut attacks) = data;

        for (ent, intent) in (&entities, &attacks).join() {
            let trait_list = move_type::get_intent_traits(&intent);

            for att_trait in trait_list {
                match att_trait {
                    crate::AttackTrait::Knockback { amount } => {
                        if let Some(ent_pos) = positions.get(ent) {
                            let event = crate::EventType::Push {
                                source_pos: rltk::Point::new(ent_pos.x, ent_pos.y),
                                amount,
                            };
                            let range = &move_type::get_attack_shape(&intent.main);
                            crate::add_event(
                                &event,
                                Some(*intent),
                                Some(ent),
                                range,
                                intent.loc,
                                false,
                            );
                        }
                    }
                    crate::AttackTrait::Damage => {
                        crate::add_damage_event(intent, Some(ent), true);
                    }
                    crate::AttackTrait::Movement => {
                        let event = crate::EventType::Movement;
                        let range = &move_type::get_attack_shape(&intent.main);
                        crate::add_event(&event, Some(*intent), Some(ent), range, intent.loc, false)
                    }
                    crate::AttackTrait::Draw { amount } => {
                        if ent == *player {
                            // TODO: enemies don't have a deck for now
                            for _ in 0..amount {
                                deck.draw();
                            }
                        }
                    }
                    crate::AttackTrait::Modifier => {
                        // this is just a marker, modified attacks don't do anything special (yet?)
                    }
                }
            }
        }

        attacks.clear();
    }
}
