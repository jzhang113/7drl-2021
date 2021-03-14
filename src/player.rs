use super::{Map, MoveIntent, Player, Position, RunState, State, Viewable, Viewshed};
use rltk::{Algorithm2D, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

fn try_move_player(ecs: &mut World, dx: i32, dy: i32) -> RunState {
    use std::cmp::{max, min};
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut movements = ecs.write_storage::<MoveIntent>();
    let map = ecs.fetch::<Map>();
    let player = ecs.fetch::<Entity>();

    for (_player, pos) in (&players, &mut positions).join() {
        let dest_index = map.get_index(pos.x + dx, pos.y + dy);

        let new_x = min(map.width, max(0, pos.x + dx));
        let new_y = min(map.height, max(0, pos.y + dy));

        if !map.blocked_tiles[dest_index] {
            let new_move = MoveIntent {
                loc: Point::new(new_x, new_y),
            };
            movements
                .insert(*player, new_move)
                .expect("Failed to insert new movement from player");

            let mut intents = ecs.fetch_mut::<crate::IntentData>();
            intents.prev_incoming_intent = None;
            intents.prev_outgoing_intent = None;

            return RunState::Running;
        } else if map.tiles[dest_index] != crate::TileType::Wall {
            // TODO: implement push
            let mut log = ecs.fetch_mut::<crate::gamelog::GameLog>();
            log.entries
                .push(format!("You can't make it through this way"));

            return RunState::AwaitingInput;
        }
    }

    RunState::AwaitingInput
}

fn select_card(
    gs: &mut State,
    index: usize,
    is_reaction: bool,
    reaction_target: Option<Entity>,
) -> RunState {
    let mut deck = gs.ecs.fetch_mut::<crate::deck::Deck>();
    let positions = gs.ecs.read_storage::<Position>();
    let player = gs.ecs.fetch::<Entity>();
    let player_pos = positions
        .get(*player)
        .expect("player didn't have a position");
    let player_point = Point::new(player_pos.x, player_pos.y);

    // don't process the input if the selection doesn't exist
    if deck.hand.len() <= index {
        return RunState::AwaitingInput;
    }

    deck.selected = index as i32;
    let attack_type = deck.hand[index];
    let mut ignore_targetting = false;

    // if we are counter attacking, only allow moves that can hit
    // unselect the card if we end up quitting
    if is_reaction {
        match reaction_target {
            None => {
                deck.selected = -1;
                return RunState::AwaitingInput;
            }
            Some(target) => {
                if let Some(target_pos) = positions.get(target) {
                    let target_point = Point::new(target_pos.x, target_pos.y);

                    match crate::move_type::is_attack_valid(
                        &attack_type,
                        player_point,
                        target_point,
                    ) {
                        None => {
                            deck.selected = -1;
                            return RunState::AwaitingInput;
                        }
                        Some(point) => {
                            // TODO: other points in range are still valid, but maybe they shouldn't be
                            gs.cursor = point;
                            gs.tab_index = 0;
                        }
                    }
                }
            }
        }
    } else {
        let shape = crate::move_type::get_attack_shape(&attack_type);
        let range_type = crate::move_type::get_attack_range(&attack_type);
        let tiles_in_range = crate::range_type::resolve_range_at(&range_type, player_point);

        // empty-shaped moves are not targetted
        if shape == crate::RangeType::Empty {
            // TODO: should this skip targetting
            ignore_targetting = true;
        }

        // update targetting specific state
        let players = gs.ecs.read_storage::<Player>();
        let positions = gs.ecs.read_storage::<Position>();
        let viewsheds = gs.ecs.read_storage::<Viewshed>();
        let map = gs.ecs.fetch::<Map>();

        for (_player, pos, viewshed) in (&players, &positions, &viewsheds).join() {
            let mut tab_targets = Vec::new();

            // We can target visible tiles in range
            for idx in viewshed.visible.iter() {
                if tiles_in_range.contains(idx) {
                    let index = map.point2d_to_index(*idx);

                    if map.blocked_tiles[index] && map.tiles[index] != crate::TileType::Wall {
                        tab_targets.push(*idx);
                    }
                }
            }

            let init_point;
            if tab_targets.len() > 0 {
                init_point = tab_targets[0];
            } else {
                init_point = Point::new(pos.x, pos.y);
            }

            gs.cursor = init_point;
            gs.tab_targets = tab_targets;
            gs.tab_index = 0;
        }
    }

    RunState::Targetting {
        attack_type,
        ignore_targetting,
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let (is_reaction, target) = {
        let can_act = gs.ecs.read_storage::<super::CanActFlag>();
        let player = gs.ecs.fetch::<Entity>();
        let player_can_act = can_act
            .get(*player)
            .expect("player_input called, but it is not your turn");

        (player_can_act.is_reaction, player_can_act.reaction_target)
    };

    handle_keys(gs, ctx, is_reaction, target)
}

pub fn end_turn_cleanup(ecs: &mut World) {
    let is_reaction = {
        let can_act = ecs.read_storage::<super::CanActFlag>();
        let player = ecs.fetch::<Entity>();
        can_act
            .get(*player)
            .expect("player_input called, but it is not your turn")
            .is_reaction
    };

    update_reaction_state(ecs, is_reaction);
    clear_lingering_cards(ecs);
}

// if we are in a reaction, remove the CanReact flag
// otherwise, we are on the main turn, so restore the flag
fn update_reaction_state(ecs: &mut World, _is_reaction: bool) {
    // let player = ecs.fetch::<Entity>();
    let mut can_act = ecs.write_storage::<super::CanActFlag>();
    // let mut can_react = ecs.write_storage::<super::CanReactFlag>();

    // if is_reaction {
    //     can_react.remove(*player);
    // } else {
    //     can_react
    //         .insert(*player, super::CanReactFlag {})
    //         .expect("Failed to insert CanReactFlag");
    // }

    can_act.clear();
}

fn clear_lingering_cards(ecs: &mut World) {
    let mut cards = ecs.write_storage::<super::CardLifetime>();
    cards.clear();
}

fn handle_keys(
    gs: &mut State,
    ctx: &mut Rltk,
    is_reaction: bool,
    reaction_target: Option<Entity>,
) -> RunState {
    match ctx.key {
        None => RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                if is_reaction {
                    return RunState::AwaitingInput;
                } else {
                    return try_move_player(&mut gs.ecs, -1, 0);
                }
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                if is_reaction {
                    return RunState::AwaitingInput;
                } else {
                    return try_move_player(&mut gs.ecs, 1, 0);
                }
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                if is_reaction {
                    return RunState::AwaitingInput;
                } else {
                    return try_move_player(&mut gs.ecs, 0, -1);
                }
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                if is_reaction {
                    return RunState::AwaitingInput;
                } else {
                    return try_move_player(&mut gs.ecs, 0, 1);
                }
            }
            VirtualKeyCode::V => RunState::ViewEnemy { index: 0 },
            VirtualKeyCode::Space => {
                let mut intents = gs.ecs.fetch_mut::<crate::IntentData>();
                intents.hidden = false;

                if !is_reaction {
                    let mut deck = gs.ecs.fetch_mut::<crate::deck::Deck>();
                    deck.draw();
                }

                RunState::Running
            }
            VirtualKeyCode::Key1 => select_card(gs, 0, is_reaction, reaction_target),
            VirtualKeyCode::Key2 => select_card(gs, 1, is_reaction, reaction_target),
            VirtualKeyCode::Key3 => select_card(gs, 2, is_reaction, reaction_target),
            VirtualKeyCode::Key4 => select_card(gs, 3, is_reaction, reaction_target),
            VirtualKeyCode::Key5 => select_card(gs, 4, is_reaction, reaction_target),
            VirtualKeyCode::Key6 => select_card(gs, 5, is_reaction, reaction_target),
            VirtualKeyCode::Key7 => select_card(gs, 6, is_reaction, reaction_target),
            _ => RunState::AwaitingInput,
        },
    }
}

pub enum SelectionResult {
    Selected,
    Canceled,
    NoResponse,
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut Rltk,
    tiles_in_range: Vec<Point>,
    ignore_targetting: bool,
) -> (SelectionResult, Option<Point>) {
    let players = gs.ecs.read_storage::<Player>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    let mut valid_target = false;

    if ignore_targetting {
        ctx.print_color(
            crate::gui::MAP_X,
            crate::gui::MAP_Y - 1,
            rltk::RGB::named(rltk::GOLD),
            rltk::RGB::named(rltk::BLACK),
            "Confirm use",
        );
    } else {
        ctx.set_active_console(0);

        // Highlight available target cells
        let mut available_cells = Vec::new();
        for (_player, viewshed) in (&players, &viewsheds).join() {
            // We have a viewshed
            for idx in viewshed.visible.iter() {
                if tiles_in_range.contains(idx) {
                    ctx.set_bg(
                        crate::gui::MAP_X + idx.x,
                        crate::gui::MAP_Y + idx.y,
                        rltk::RGB::named(rltk::BLUE),
                    );
                    available_cells.push(idx);
                }
            }
        }

        // Draw cursor
        valid_target = available_cells
            .iter()
            .any(|pos| pos.x == gs.cursor.x && pos.y == gs.cursor.y);

        let cursor_color;
        if valid_target {
            cursor_color = rltk::RGB::named(rltk::CYAN);
        } else {
            cursor_color = rltk::RGB::named(rltk::RED);
        }
        ctx.set_bg(
            crate::gui::MAP_X + gs.cursor.x,
            crate::gui::MAP_Y + gs.cursor.y,
            cursor_color,
        );
        ctx.set_active_console(1);

        if valid_target {
            ctx.print_color(
                crate::gui::MAP_X,
                crate::gui::MAP_Y - 1,
                rltk::RGB::named(rltk::GOLD),
                rltk::RGB::named(rltk::BLACK),
                "Select Target",
            );
        } else {
            ctx.print_color(
                crate::gui::MAP_X,
                crate::gui::MAP_Y - 1,
                rltk::RGB::named(rltk::DARKGOLDENROD),
                rltk::RGB::named(rltk::BLACK),
                "Invalid Target",
            );
        }
    }

    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Escape => return (SelectionResult::Canceled, None),
            VirtualKeyCode::Space | VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                if valid_target {
                    return (
                        SelectionResult::Selected,
                        Some(Point::new(gs.cursor.x, gs.cursor.y)),
                    );
                } else if ignore_targetting {
                    return (SelectionResult::Selected, None);
                } else {
                    return (SelectionResult::Canceled, None);
                }
            }
            VirtualKeyCode::Tab => {
                let length = gs.tab_targets.len();

                if length > 0 {
                    gs.tab_index += 1;
                    gs.cursor = gs.tab_targets[gs.tab_index % length];
                }
            }
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                gs.cursor.x -= 1;
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                gs.cursor.x += 1;
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                gs.cursor.y -= 1;
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                gs.cursor.y += 1;
            }
            // TODO: placeholder
            VirtualKeyCode::V => return (SelectionResult::Canceled, None),
            _ => {}
        },
    };

    (SelectionResult::NoResponse, None)
}

pub fn view_input(gs: &mut State, ctx: &mut Rltk, index: u32) -> RunState {
    let entities = gs.ecs.entities();
    let viewables = gs.ecs.read_storage::<Viewable>();

    let mut new_index = index;
    let mut max_index = 0;

    for (ent, view) in (&entities, &viewables).join() {
        if let Some(list_index) = view.list_index {
            max_index = std::cmp::max(list_index, max_index);

            if list_index == index {
                crate::gui::draw_viewable_info(&gs.ecs, ctx, &ent, index);
            }
        }
    }

    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Escape => return RunState::AwaitingInput,
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                if new_index > 0 {
                    new_index -= 1;
                } else {
                    new_index += max_index;
                }
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                new_index += 1;
            }
            _ => {}
        },
    }

    RunState::ViewEnemy {
        index: new_index % (max_index + 1),
    }
}
