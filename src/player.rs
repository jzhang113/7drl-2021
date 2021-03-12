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

fn select_card(gs: &mut State, index: usize) -> RunState {
    let mut deck = gs.ecs.fetch_mut::<crate::deck::Deck>();

    // don't process the input if the selection doesn't exist
    if deck.hand.len() <= index {
        return RunState::AwaitingInput;
    }

    deck.selected = index as i32;
    let attack_type = deck.hand[index];

    let shape = crate::move_type::get_attack_shape(&attack_type);
    let mut range = crate::move_type::get_attack_range(&attack_type);
    if let Some(modifier) = gs.attack_modifier {
        range += crate::move_type::get_attack_range(&modifier);
    }

    // empty-shaped moves are not targetted
    if shape == crate::RangeType::Empty {
        // TODO: should this skip targetting
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
            let distance = rltk::DistanceAlg::Manhattan.distance2d(Point::new(pos.x, pos.y), *idx);
            if distance <= range as f32 {
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

    return RunState::Targetting { attack_type };
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let is_reaction = {
        let can_act = gs.ecs.read_storage::<super::CanActFlag>();
        let player = gs.ecs.fetch::<Entity>();
        can_act
            .get(*player)
            .expect("player_input called, but it is not your turn")
            .is_reaction
    };

    let result = handle_keys(gs, ctx, is_reaction);

    if result == RunState::Running {
        update_hand(&mut gs.ecs);
        update_reaction_state(&mut gs.ecs, is_reaction);
        clear_lingering_cards(&mut gs.ecs);
    }

    result
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

fn update_hand(ecs: &mut World) {
    let mut deck = ecs.fetch_mut::<crate::deck::Deck>();
    deck.draw();
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

fn handle_keys(gs: &mut State, ctx: &mut Rltk, is_reaction: bool) -> RunState {
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
            VirtualKeyCode::Space => RunState::Running,
            VirtualKeyCode::Key1 => select_card(gs, 0),
            VirtualKeyCode::Key2 => select_card(gs, 1),
            VirtualKeyCode::Key3 => select_card(gs, 2),
            VirtualKeyCode::Key4 => select_card(gs, 3),
            VirtualKeyCode::Key5 => select_card(gs, 4),
            VirtualKeyCode::Key6 => select_card(gs, 5),
            VirtualKeyCode::Key7 => select_card(gs, 6),
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
    range: i32,
) -> (SelectionResult, Option<Point>) {
    let players = gs.ecs.read_storage::<Player>();
    let positions = gs.ecs.read_storage::<Position>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        crate::gui::MAP_X,
        crate::gui::MAP_Y - 1,
        rltk::RGB::named(rltk::GOLD),
        rltk::RGB::named(rltk::BLACK),
        "Select Target",
    );

    ctx.set_active_console(0);

    // Highlight available target cells
    let mut available_cells = Vec::new();
    for (_player, pos, viewshed) in (&players, &positions, &viewsheds).join() {
        // We have a viewshed
        for idx in viewshed.visible.iter() {
            let distance = rltk::DistanceAlg::Manhattan.distance2d(Point::new(pos.x, pos.y), *idx);
            if distance <= range as f32 {
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
    let valid_target = available_cells
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
