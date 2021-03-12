use super::*;
use rltk::{Algorithm2D, Rltk, RGB};

// #region UI constants
pub const MAP_X: i32 = 16;
pub const MAP_Y: i32 = 1;
pub const MAP_W: i32 = 80;
pub const MAP_H: i32 = 50;

const CARD_Y: i32 = SIDE_H;
const CARD_W: i32 = 10;
const CARD_H: i32 = 15;

const SIDE_X: i32 = 0;
const SIDE_Y: i32 = 0;
const SIDE_W: i32 = 15;
const SIDE_H: i32 = 50;

pub const CONSOLE_WIDTH: i32 = MAP_W + SIDE_W + 2;
pub const CONSOLE_HEIGHT: i32 = MAP_H + CARD_H + 1;

const SHOW_MAP: bool = false;
const SHOW_REND: bool = false;
// #endregion

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

pub fn draw_cards(ecs: &World, ctx: &mut Rltk) {
    let cards = ecs.read_storage::<CardLifetime>();
    let card_stack_active = crate::events::CARDSTACK
        .lock()
        .expect("Failed to lock CARDSTACK");

    for (i, card) in card_stack_active.iter().enumerate() {
        draw_card(card, i as i32, ctx);

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

    let mut card_stack_linger = cards.join().collect::<Vec<_>>();
    card_stack_linger.sort_by(|&a, b| a.data.offset.partial_cmp(&b.data.offset).unwrap());
    for card in card_stack_linger {
        draw_card(&card.data, card.data.offset, ctx);
    }
}

fn draw_card(card: &CardRequest, offset: i32, ctx: &mut Rltk) {
    ctx.draw_box(
        50 + 3 * offset,
        10,
        10,
        15,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print(51 + 3 * offset, 11, card.name.clone());
}

pub fn draw_hand(ecs: &World, ctx: &mut Rltk) {
    let deck = ecs.fetch::<crate::deck::Deck>();

    for xpos in 0..CONSOLE_WIDTH {
        ctx.set(
            xpos,
            CONSOLE_HEIGHT - 1,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('─'),
        )
    }
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
        CONSOLE_HEIGHT - 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┴'),
    );
    ctx.set(
        86,
        CONSOLE_HEIGHT - 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('┴'),
    );

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

        ctx.draw_box(
            start_x + (CARD_W + 1) * (i as i32),
            CARD_Y,
            CARD_W,
            CARD_H,
            fore_color,
            RGB::named(rltk::BLACK),
        );

        let attack_name = crate::move_type::get_attack_name(card);
        let xpos = start_x + 1 + (CARD_W + 1) * (i as i32);
        let ypos = CARD_Y + 4;

        ctx.print(xpos, CARD_Y + 1, format!("{}) {}", i + 1, attack_name));

        // stat values
        let power_str = format!("{}", crate::move_type::get_attack_power(card));
        let speed_str = format!("{}", crate::move_type::get_attack_speed(card));
        let guard_str = format!("{}", crate::move_type::get_attack_guard(card));
        ctx.print(xpos + 2 - (power_str.len() as i32), ypos, power_str);
        ctx.print(xpos + 5 - (speed_str.len() as i32), ypos, speed_str);
        ctx.print(xpos + 8 - (guard_str.len() as i32), ypos, guard_str);

        // stat icons
        ctx.set_active_console(2);
        ctx.set(
            xpos + 2,
            ypos,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
            1,
        );
        ctx.set(
            xpos + 5,
            ypos,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            2,
        );
        ctx.set(
            xpos + 8,
            ypos,
            RGB::named(rltk::LIGHTBLUE),
            RGB::named(rltk::BLACK),
            0,
        );
        ctx.set_active_console(1);
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
    let renderables = ecs.read_storage::<Renderable>();
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
    ctx.print(SIDE_X + 1, SIDE_Y, format!("Seen"));

    let x = SIDE_X + 1;
    let mut y = SIDE_Y + 2;

    for (rend, pos, health) in (&renderables, &positions, &healths).join() {
        if !player_view
            .visible
            .iter()
            .any(|view_pos| view_pos.x == pos.x && view_pos.y == pos.y)
        {
            continue;
        }

        ctx.print(x, y, format!("{}:", rend.symbol as u8 as char));
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
