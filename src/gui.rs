use super::*;
use rltk::{Algorithm2D, Rltk, RGB};

// #region UI constants
pub const MAP_X: i32 = SIDE_W + 1;
pub const MAP_Y: i32 = 1;
pub const MAP_W: i32 = 79;
pub const MAP_H: i32 = 50;

const CARD_Y: i32 = SIDE_H;
const CARD_W: i32 = 10;
const CARD_H: i32 = 15;

const SIDE_X: i32 = 0;
const SIDE_Y: i32 = 0;
const SIDE_W: i32 = 16;
const SIDE_H: i32 = 50;

pub const CONSOLE_WIDTH: i32 = MAP_W + SIDE_W + 2;
pub const CONSOLE_HEIGHT: i32 = MAP_H + CARD_H + 2;

const SHOW_MAP: bool = false;
const SHOW_REND: bool = false;
// #endregion

struct AttackData {
    name: String,
    power: i32,
    speed: i32,
    guard: i32,
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        MAP_X - 1,
        MAP_Y - 1,
        MAP_W + 1,
        MAP_H + 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let map = ecs.fetch::<Map>();

    let mut x = 0;
    let mut y = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.known_tiles[idx] || SHOW_MAP {
            let symbol;
            let mut fg;

            match tile {
                TileType::Floor => {
                    symbol = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    symbol = rltk::to_cp437('#');
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
            }

            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }
            ctx.set(MAP_X + x, MAP_Y + y, fg, RGB::from_f32(0., 0., 0.), symbol);
        }

        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}

pub fn draw_renderables(ecs: &World, ctx: &mut Rltk) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let particles = ecs.read_storage::<ParticleLifetime>();
    let map = ecs.fetch::<Map>();

    for (pos, render, particle) in (&positions, &renderables, (&particles).maybe()).join() {
        if let Some(lifetime) = particle {
            let mut fg = render.fg;
            let mut bg = render.bg;

            if lifetime.should_fade {
                let fade_percent = ezing::expo_inout(1.0 - lifetime.remaining / lifetime.base);
                let base_color = RGB::named(rltk::BLACK);

                fg = fg.lerp(base_color, fade_percent);
                bg = bg.lerp(base_color, fade_percent);
            }

            ctx.set_active_console(0);
            ctx.set(MAP_X + pos.x, MAP_Y + pos.y, fg, bg, render.symbol);
            ctx.set_active_console(1);
        } else if map.visible_tiles[map.get_index(pos.x, pos.y)] || SHOW_REND {
            ctx.set(
                MAP_X + pos.x,
                MAP_Y + pos.y,
                render.fg,
                render.bg,
                render.symbol,
            );
        }
    }
}

pub fn draw_cards(_ecs: &World, ctx: &mut Rltk) {
    let card_stack_active = crate::events::CARDSTACK
        .lock()
        .expect("Failed to lock CARDSTACK");

    for card in card_stack_active.iter() {
        ctx.set_active_console(0);
        for pos in card.affected.iter() {
            ctx.set(
                MAP_X + pos.x,
                MAP_Y + pos.y,
                RGB::named(rltk::RED),
                RGB::named(rltk::BLACK),
                rltk::to_cp437('█'),
            );
        }
        ctx.set_active_console(1);
    }
}

pub fn draw_intents(ecs: &World, ctx: &mut Rltk) {
    let intents = ecs.fetch::<IntentData>();

    ctx.print(4, 13, "INCOMING");
    if let Some(incoming) = intents.prev_incoming_intent {
        if intents.hidden {
            draw_card_hidden(ctx, 3, 14, RGB::named(rltk::WHITE));
        } else {
            let data = AttackData {
                name: crate::move_type::get_intent_name(&incoming),
                power: crate::move_type::get_intent_power(&incoming),
                speed: crate::move_type::get_intent_speed(&incoming),
                guard: crate::move_type::get_intent_guard(&incoming),
            };

            draw_card(ctx, data, 3, 14, RGB::named(rltk::WHITE));
        }
    }

    ctx.print(4, 32, "OUTGOING");
    if let Some(outgoing) = intents.prev_outgoing_intent {
        let data = AttackData {
            name: crate::move_type::get_intent_name(&outgoing),
            power: crate::move_type::get_intent_power(&outgoing),
            speed: crate::move_type::get_intent_speed(&outgoing),
            guard: crate::move_type::get_intent_guard(&outgoing),
        };

        draw_card(ctx, data, 3, 33, RGB::named(rltk::WHITE));
    }
}

fn draw_card_hidden(ctx: &mut Rltk, x_start: i32, y_start: i32, fore_color: RGB) {
    ctx.draw_box(
        x_start,
        y_start,
        CARD_W,
        CARD_H,
        fore_color,
        RGB::named(rltk::BLACK),
    );

    ctx.print(x_start + 1, y_start + 1, "???");
}

fn draw_card(ctx: &mut Rltk, attack: AttackData, x_start: i32, y_start: i32, fore_color: RGB) {
    ctx.draw_box(
        x_start,
        y_start,
        CARD_W,
        CARD_H,
        fore_color,
        RGB::named(rltk::BLACK),
    );

    ctx.print(x_start + 1, y_start + 1, attack.name);

    let y_stats = y_start + 3;

    // stat values
    let power_str = format!("{}", attack.power);
    let speed_str = format!("{}", attack.speed);
    let guard_str = format!("{}", attack.guard);
    ctx.print(x_start + 3 - (power_str.len() as i32), y_stats, power_str);
    ctx.print(x_start + 6 - (speed_str.len() as i32), y_stats, speed_str);
    ctx.print(x_start + 9 - (guard_str.len() as i32), y_stats, guard_str);

    // stat icons
    ctx.set_active_console(2);
    ctx.set(
        x_start + 3,
        y_stats,
        RGB::named(rltk::RED),
        RGB::named(rltk::BLACK),
        1,
    );
    ctx.set(
        x_start + 6,
        y_stats,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        2,
    );
    ctx.set(
        x_start + 9,
        y_stats,
        RGB::named(rltk::LIGHTBLUE),
        RGB::named(rltk::BLACK),
        0,
    );
    ctx.set_active_console(1);
}

pub fn draw_hand(ecs: &World, ctx: &mut Rltk) {
    let deck = ecs.fetch::<crate::deck::Deck>();

    ctx.draw_box(
        SIDE_X,
        SIDE_Y + SIDE_H + 1,
        10,
        14,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.draw_box(
        86,
        SIDE_Y + SIDE_H + 1,
        10,
        14,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    // fix breaks
    ctx.set(
        SIDE_X,
        SIDE_Y + SIDE_H + 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('├'),
    );
    ctx.set(
        10,
        SIDE_Y + SIDE_H + 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┬'),
    );
    ctx.set(
        MAP_X - 1,
        MAP_Y - 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┬'),
    );
    ctx.set(
        MAP_X - 1,
        MAP_Y + SIDE_H,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┴'),
    );
    ctx.set(
        86,
        MAP_Y + SIDE_H,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┬'),
    );
    ctx.set(
        CONSOLE_WIDTH - 1,
        MAP_Y + SIDE_H,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┤'),
    );
    ctx.set(
        10,
        CONSOLE_HEIGHT - 2,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┴'),
    );
    ctx.set(
        86,
        CONSOLE_HEIGHT - 2,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┴'),
    );
    for xpos in 11..86 {
        ctx.set(
            xpos,
            CONSOLE_HEIGHT - 2,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('─'),
        )
    }

    ctx.print(
        2,
        CARD_Y + 8,
        pluralize("card".to_string(), deck.cards_remaining()),
    );
    ctx.print(1, CARD_Y + 9, format!("remaining"));

    ctx.print(
        88,
        CARD_Y + 8,
        pluralize("card".to_string(), deck.cards_discarded()),
    );
    ctx.print(87, CARD_Y + 9, format!("discarded"));

    let hand_size = deck.hand.len() as i32;
    let start_x = (CONSOLE_WIDTH - hand_size * (CARD_W + 1)) / 2;

    for (i, card) in deck.hand.iter().enumerate() {
        let index = i as i32;
        let fore_color;
        if index == deck.selected {
            fore_color = RGB::named(rltk::GOLD);
        } else {
            fore_color = RGB::from_hex("#AFE0CE").unwrap();
        }

        let xpos = start_x + (CARD_W + 1) * (i as i32);
        let ypos = CARD_Y;

        let data = AttackData {
            name: format!("{}) {}", i + 1, crate::move_type::get_attack_name(card)),
            power: crate::move_type::get_attack_power(card),
            speed: crate::move_type::get_attack_speed(card),
            guard: crate::move_type::get_attack_guard(card),
        };

        draw_card(ctx, data, xpos, ypos, fore_color);
    }
}

fn pluralize(root: String, count: i32) -> String {
    if count != 1 {
        return format!("{} {}s", count, root);
    } else {
        return format!("1 {}", root);
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let healths = ecs.read_storage::<Health>();
    let mut viewables = ecs.write_storage::<Viewable>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let positions = ecs.read_storage::<Position>();

    let player = ecs.fetch::<Entity>();
    let player_view = viewsheds
        .get(*player)
        .expect("Player didn't have a viewshed");

    ctx.draw_box(
        SIDE_X,
        SIDE_Y,
        SIDE_W,
        SIDE_H + 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let x = SIDE_X + 1;
    let mut y = SIDE_Y + 1;
    let mut index = 0;

    for (mut view, pos, health) in (&mut viewables, &positions, &healths).join() {
        if !player_view
            .visible
            .iter()
            .any(|view_pos| view_pos.x == pos.x && view_pos.y == pos.y)
        {
            continue;
        }

        ctx.print(x, y, format!("{}:", view.symbol as u8 as char));
        view.list_index = Some(index);
        let curr_hp = std::cmp::max(0, health.current);

        for i in 0..curr_hp {
            ctx.set(
                x + i + 2,
                y,
                RGB::named(rltk::RED),
                RGB::named(rltk::BLACK),
                rltk::to_cp437('o'),
            );
        }

        for i in curr_hp..health.max {
            ctx.set(
                x + i + 2,
                y,
                RGB::named(rltk::DARKRED),
                RGB::named(rltk::BLACK),
                rltk::to_cp437('o'),
            );
        }

        y += 2;
        index += 1;
        if index > 5 {
            break;
        }

        // TODO: what to do with excess?
    }

    // ctx.draw_box(
    //     0,
    //     50,
    //     79,
    //     6,
    //     RGB::named(rltk::WHITE),
    //     RGB::named(rltk::BLACK),
    // );

    // let log = ecs.fetch::<super::gamelog::GameLog>();
    // for (line, message) in log.entries.iter().rev().take(5).enumerate() {
    //     ctx.print(2, 50 + line + 1, message);
    // }

    // ctx.print(74, 1, format!("{} fps", ctx.fps));
    // draw_tooltips(ecs, ctx);
}

pub fn update_controls_text(ecs: &World, ctx: &mut Rltk, status: &RunState) {
    ctx.set_active_console(3);
    ctx.cls();

    let x = 0;
    let y = CONSOLE_HEIGHT - 1;
    let icon_color = RGB::named(rltk::GOLD);
    let bg_color = RGB::named(rltk::BLACK);

    // movement controls
    let move_section_x = x;
    ctx.set(move_section_x + 1, y, icon_color, bg_color, 27);
    ctx.set(move_section_x + 2, y, icon_color, bg_color, 25);
    ctx.set(move_section_x + 3, y, icon_color, bg_color, 24);
    ctx.set(move_section_x + 4, y, icon_color, bg_color, 26);
    ctx.print(move_section_x + 6, y, "move");

    match *status {
        RunState::AwaitingInput => {
            let is_reaction = {
                let can_act = ecs.read_storage::<super::CanActFlag>();
                let player = ecs.fetch::<Entity>();
                can_act
                    .get(*player)
                    .expect("uh-oh, we're waiting for input but the player can't act")
                    .is_reaction
            };

            // examine
            let view_section_x = 13;
            ctx.print_color(view_section_x, y, icon_color, bg_color, "v");
            ctx.print(view_section_x + 1, y, "iew map");

            // space bar
            let space_section_x = 25;
            let space_action_str;
            if is_reaction {
                space_action_str = "brace";
            } else {
                space_action_str = "recover";
            }

            ctx.print_color(space_section_x, y, icon_color, bg_color, "[SPACE]");
            ctx.print(space_section_x + 8, y, space_action_str);

            // card section
            let card_section_x = 43;
            ctx.print_color(card_section_x, y, icon_color, bg_color, "[1-7]");
            ctx.print(card_section_x + 6, y, "use card");
        }
        RunState::Targetting { .. } => {
            // examine
            let view_section_x = 13;
            ctx.print_color(view_section_x, y, icon_color, bg_color, "v");
            ctx.print(view_section_x + 1, y, "iew card");

            // space bar
            let space_section_x = 25;
            ctx.print_color(space_section_x, y, icon_color, bg_color, "[SPACE]");
            ctx.print(space_section_x + 8, y, "confirm");

            // escape
            let escape_section_x = 43;
            ctx.print_color(escape_section_x, y, icon_color, bg_color, "[ESC]");
            ctx.print(escape_section_x + 6, y, "cancel");

            // tab target
            let tab_section_x = 58;
            ctx.print_color(tab_section_x, y, icon_color, bg_color, "[TAB]");
            ctx.print(tab_section_x + 6, y, "next target");
        }
        RunState::ViewEnemy { .. } => {
            // escape
            let escape_section_x = 13;
            ctx.print_color(escape_section_x, y, icon_color, bg_color, "[ESC]");
            ctx.print(escape_section_x + 6, y, "cancel");
        }
        _ => {}
    }

    ctx.set_active_console(1);
}

pub fn draw_viewable_info(ecs: &World, ctx: &mut Rltk, entity: &Entity, index: u32) {
    let selected_color = RGB::named(rltk::GOLD);
    let bg_color = RGB::named(rltk::BLACK);

    ctx.set(
        0,
        2 * index + 1,
        RGB::named(rltk::GOLD),
        bg_color,
        rltk::to_cp437('>'),
    );

    let positions = ecs.read_storage::<Position>();
    let viewables = ecs.read_storage::<Viewable>();
    let healths = ecs.read_storage::<Health>();

    let pos = positions
        .get(*entity)
        .expect("viewable didn't have a position");
    let view = viewables.get(*entity).expect("viewable didn't have a view");
    let health = healths.get(*entity).expect("viewable didn't have health");

    let x = MAP_X + pos.x;
    let y = MAP_Y + pos.y;

    ctx.set_active_console(0);
    ctx.set(x, y, selected_color, bg_color, rltk::to_cp437('█'));
    ctx.set_active_console(1);

    let (box_x, box_y) = position_box(ctx, x, y, 10, 10, selected_color, bg_color);

    ctx.print(box_x + 1, box_y, view.name.clone());
    ctx.print(
        box_x + 1,
        box_y + 1,
        format!("HP: {}/{}", health.current, health.max),
    );
}

// draw a box stemming from a given point
// returns the top left of the new box
fn position_box(ctx: &mut Rltk, x: i32, y: i32, w: i32, h: i32, fg: RGB, bg: RGB) -> (i32, i32) {
    let right = x + w < CONSOLE_WIDTH - 1;
    let down = y + h < MAP_H;

    // boxes prefer to be right and down if several positions are possible
    if right {
        if down {
            ctx.draw_box(x + 1, y, w, h, fg, bg);
            ctx.set(x + 1, y, fg, bg, rltk::to_cp437('┬'));
            return (x + 1, y);
        } else {
            ctx.draw_box(x + 1, y - h, w, h, fg, bg);
            ctx.set(x + 1, y, fg, bg, rltk::to_cp437('┴'));
            return (x + 1, y - h);
        }
    } else {
        if down {
            ctx.draw_box(x - w - 1, y, w, h, fg, bg);
            ctx.set(x - 1, y, fg, bg, rltk::to_cp437('┬'));
            return (x - w - 1, y);
        } else {
            ctx.draw_box(x - w - 1, y - h, w, h, fg, bg);
            ctx.set(x - 1, y, fg, bg, rltk::to_cp437('┴'));
            return (x - w - 1, y - h);
        }
    }
}

// TODO
fn _draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let renderables = ecs.read_storage::<Renderable>();
    let positions = ecs.read_storage::<Position>();

    let mouse_point = ctx.mouse_point();
    if !map.in_bounds(mouse_point) {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();

    for (rend, pos) in (&renderables, &positions).join() {
        if pos.as_point() == mouse_point {
            tooltip.push(rend.symbol.to_string());
        }
    }

    if !tooltip.is_empty() {
        // placeholder
        ctx.print_color(
            1,
            1,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::GREY),
            tooltip.first().unwrap(),
        );
    }
}
