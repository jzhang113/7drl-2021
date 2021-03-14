use super::{AiState, AttackIntent, CanActFlag, Map, MoveIntent, Moveset, Position, Viewshed};
use rltk::{Algorithm2D, BaseMap};
use specs::prelude::*;

pub enum Behavior {
    Sleep,
    Wander,
    Chase,
    Flee,
}

pub struct AiSystem;

impl<'a> System<'a> for AiSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CanActFlag>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MoveIntent>,
        WriteStorage<'a, AttackIntent>,
        WriteStorage<'a, AiState>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Moveset>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut can_act,
            positions,
            mut moves,
            mut attacks,
            mut states,
            viewsheds,
            movesets,
            map,
            player,
            mut rng,
        ) = data;
        let mut turn_done = Vec::new();
        let player_pos = positions.get(*player).unwrap();

        for (ent, _turn, pos, state, viewshed, moveset) in (
            &entities,
            &can_act,
            &positions,
            &mut states,
            &viewsheds,
            &movesets,
        )
            .join()
        {
            let curr_index = map.get_index(pos.x, pos.y);
            let can_see_player = viewshed
                .visible
                .iter()
                .any(|pos| pos.x == player_pos.x && pos.y == player_pos.y);

            match state.status {
                Behavior::Sleep => {
                    // the do nothing state
                    // TODO: trigger wake up
                }
                Behavior::Wander => {
                    if can_see_player {
                        state.status = Behavior::Chase;
                        state.tracking = Some(rltk::Point::new(player_pos.x, player_pos.y));
                    } else {
                        // pick a random tile we can move to
                        let exits = map.get_available_exits(curr_index);
                        let exit_index = rng.range(0, exits.len());
                        let chosen_exit = exits[exit_index].0;
                        let movement = MoveIntent {
                            loc: map.index_to_point2d(chosen_exit),
                        };

                        moves
                            .insert(ent, movement)
                            .expect("Failed to insert movement from AI");
                    }
                }
                Behavior::Chase => {
                    if can_see_player {
                        // check if we have any attacks that can hit
                        let mut attack = None;
                        let orig_point = rltk::Point::new(pos.x, pos.y);
                        let player_point = rltk::Point::new(player_pos.x, player_pos.y);

                        // track the player's current position
                        state.tracking = Some(player_point);

                        for potential_attack in moveset.moves.iter() {
                            if crate::move_type::is_attack_valid(
                                potential_attack,
                                orig_point,
                                player_point,
                            )
                            .is_some()
                            {
                                attack = Some(potential_attack);
                                break;
                            }
                        }

                        match attack {
                            None => {
                                // if we can't hit, just move towards the player
                                let curr_index = map.get_index(pos.x, pos.y);
                                let player_index = map.get_index(player_pos.x, player_pos.y);
                                let movement = move_towards(&*map, curr_index, player_index);

                                match movement {
                                    None => {
                                        // we shouldn't really be here, just give up chasing I guess
                                        state.status = Behavior::Wander;
                                        state.tracking = None;
                                    }
                                    Some(movement) => {
                                        moves
                                            .insert(ent, movement)
                                            .expect("Failed to insert movement from AI");
                                    }
                                }
                            }
                            Some(attack) => {
                                let intent =
                                    crate::move_type::get_attack_intent(attack, player_point, None);

                                attacks
                                    .insert(ent, intent)
                                    .expect("Failed to insert attack from AI");
                            }
                        }
                    } else {
                        match state.tracking {
                            None => {
                                // we don't have anything to chase, return to wander
                                state.status = Behavior::Wander;
                                state.tracking = None;
                            }
                            Some(target_point) => {
                                let target_index = map.point2d_to_index(target_point);
                                let movement = move_towards(&*map, curr_index, target_index);

                                match movement {
                                    None => {
                                        // most likely reason we got here is because we reached the target point
                                        // if we didn't see the player on the way, return to wandering
                                        state.status = Behavior::Wander;
                                        state.tracking = None;
                                    }
                                    Some(movement) => {
                                        moves
                                            .insert(ent, movement)
                                            .expect("Failed to insert movement from AI");
                                    }
                                }
                            }
                        }
                    }
                }
                Behavior::Flee => {
                    // TODO
                }
            }

            turn_done.push(ent);
        }

        for done in turn_done.iter() {
            can_act.remove(*done);
        }
    }
}

fn move_towards(map: &Map, curr_index: usize, target_index: usize) -> Option<MoveIntent> {
    let path = rltk::a_star_search(curr_index, target_index, &*map);

    if path.success && path.steps.len() > 1 {
        let next_pos = map.index_to_point2d(path.steps[1]);
        return Some(MoveIntent { loc: next_pos });
    } else {
        return None;
    }
}
