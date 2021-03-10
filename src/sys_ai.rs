use super::{AttackIntent, CanActFlag, Map, MoveIntent, Moveset, Position, Viewshed};
use rltk::Algorithm2D;
use specs::prelude::*;

pub struct AiSystem;

impl<'a> System<'a> for AiSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CanActFlag>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MoveIntent>,
        WriteStorage<'a, AttackIntent>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Moveset>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut can_act,
            positions,
            mut moves,
            mut attacks,
            viewsheds,
            movesets,
            map,
            player,
        ) = data;
        let mut turn_done = Vec::new();
        let player_pos = positions.get(*player).unwrap();

        for (ent, _turn, pos, viewshed, moveset) in (
            &entities,
            &can_act,
            &positions,
            &viewsheds,
            (&movesets).maybe(),
        )
            .join()
        {
            if ent == *player {
                // player turn, handled in player.rs
                continue;
            }

            if viewshed
                .visible
                .iter()
                .any(|pos| pos.x == player_pos.x && pos.y == player_pos.y)
            {
                if let Some(moveset) = moveset {
                    for potential_attack in moveset.moves.iter() {
                        crate::move_type::is_attack_valid(
                            potential_attack,
                            rltk::Point::new(pos.x, pos.y),
                        )
                    }
                } else {
                    // if we can see the player move towards them
                    let curr_index = map.get_index(pos.x, pos.y);
                    let player_index = map.get_index(player_pos.x, player_pos.y);
                    let path = rltk::a_star_search(curr_index, player_index, &*map);
                    let next_pos = map.index_to_point2d(path.steps[1]);
                    if next_pos.x == player_pos.x && next_pos.y == player_pos.y {
                        let attack = AttackIntent {
                            loc: next_pos,
                            range: crate::RangeType::Single,
                        };
                        attacks
                            .insert(ent, attack)
                            .expect("Failed to insert attack from AI");
                    } else {
                        let movement = MoveIntent { loc: next_pos };
                        moves
                            .insert(ent, movement)
                            .expect("Failed to insert movement from AI");
                    }
                }
            } else {
                // else wait for now
            }

            turn_done.push(ent);
        }

        for done in turn_done.iter() {
            can_act.remove(*done);
        }
    }
}
