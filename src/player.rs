use super::{AttackIntent, Map, MoveIntent, Player, Position, RunState, State};
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;

fn try_move_player(ecs: &mut World, dx: i32, dy: i32) -> RunState {
    use std::cmp::{max, min};
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut movements = ecs.write_storage::<MoveIntent>();
    let mut attacks = ecs.write_storage::<AttackIntent>();
    let map = ecs.fetch::<Map>();
    let player = ecs.fetch::<Entity>();

    for (_player, pos) in (&players, &mut positions).join() {
        let dest_index = map.get_index(pos.x + dx, pos.y + dy);

        let new_x = min(map.width, max(0, pos.x + dx));
        let new_y = min(map.height, max(0, pos.y + dy));

        if !map.blocked_tiles[dest_index] {
            let new_move = MoveIntent {
                loc: rltk::Point::new(new_x, new_y),
            };
            movements
                .insert(*player, new_move)
                .expect("Failed to insert new movement from player");

            return RunState::Running;
        } else if map.tiles[dest_index] != crate::TileType::Wall {
            let new_attack = AttackIntent {
                loc: rltk::Point::new(new_x, new_y),
                range: crate::RangeType::Single,
            };
            attacks
                .insert(*player, new_attack)
                .expect("Failed to insert new attack from player");

            return RunState::Running;
        }
    }

    RunState::AwaitingInput
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let is_reaction: bool;
    {
        // we expect it to be our turn
        let can_act = gs.ecs.read_storage::<super::CanActFlag>();
        let player = gs.ecs.fetch::<Entity>();
        is_reaction = can_act
            .get(*player)
            .expect("player_input called, but it is not your turn")
            .is_reaction;
    }

    let result = handle_keys(gs, ctx, is_reaction);

    if result == RunState::Running {
        update_reaction_state(&mut gs.ecs, is_reaction);
        clear_lingering_cards(&mut gs.ecs);
    }

    result
}

// if we are in a reaction, remove the CanReact flag
// otherwise, we are on the main turn, so restore the flag
fn update_reaction_state(ecs: &mut World, is_reaction: bool) {
    let player = ecs.fetch::<Entity>();
    let mut can_act = ecs.write_storage::<super::CanActFlag>();
    let mut can_react = ecs.write_storage::<super::CanReactFlag>();

    if is_reaction {
        can_react.remove(*player);
    } else {
        can_react
            .insert(*player, super::CanReactFlag {})
            .expect("Failed to insert CanReactFlag");
    }

    can_act.clear();
}

fn clear_lingering_cards(ecs: &mut World) {
    let mut cards = ecs.write_storage::<super::CardLifetime>();
    cards.clear();
}

fn handle_keys(gs: &mut State, ctx: &mut Rltk, _is_reaction: bool) -> RunState {
    match ctx.key {
        None => RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(&mut gs.ecs, -1, 0)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(&mut gs.ecs, 1, 0)
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(&mut gs.ecs, 0, -1)
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(&mut gs.ecs, 0, 1)
            }
            _ => RunState::AwaitingInput,
        },
    }
}
