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
                println!("{:?} sees the player and yells", ent);

                if let Some(moveset) = moveset {
                    let mut attack = None;
                    let orig_point = rltk::Point::new(pos.x, pos.y);
                    let player_point = rltk::Point::new(player_pos.x, player_pos.y);

                    for potential_attack in moveset.moves.iter() {
                        if crate::move_type::is_attack_valid(
                            potential_attack,
                            orig_point,
                            player_point,
                        ) {
                            attack = Some(potential_attack);
                            break;
                        }
                    }

                    match attack {
                        None => {
                            let curr_index = map.get_index(pos.x, pos.y);
                            let player_index = map.get_index(player_pos.x, player_pos.y);
                            let movement = move_towards(&*map, curr_index, player_index);
                            moves
                                .insert(ent, movement)
                                .expect("Failed to insert movement from AI");
                        }
                        Some(attack) => {
                            let intent = crate::move_type::get_attack_intent(attack, player_point);

                            attacks
                                .insert(ent, intent)
                                .expect("Failed to insert attack from AI");
                        }
                    }
                }

                turn_done.push(ent);
            }
        }

        for done in turn_done.iter() {
            can_act.remove(*done);
        }
    }
}

fn move_towards(map: &Map, curr_index: usize, target_index: usize) -> MoveIntent {
    let path = rltk::a_star_search(curr_index, target_index, &*map);
    let next_pos = map.index_to_point2d(path.steps[1]);

    MoveIntent { loc: next_pos }
}
