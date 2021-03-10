use super::{
    CardLifetime, CardRequest, Health, Map, ParticleLifetime, Position, Renderable, TileType,
};
use rltk::{Algorithm2D, Rltk, RGB};
use specs::prelude::*;

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut x = 0;
    let mut y = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.known_tiles[idx] {
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
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), symbol);
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
            ctx.set(pos.x, pos.y, fg, bg, render.symbol);
            ctx.set_active_console(1);
        } else if map.visible_tiles[map.get_index(pos.x, pos.y)] {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.symbol);
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
                pos.x,
                pos.y,
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

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let health = ecs.read_storage::<Health>();
    let player = ecs.fetch::<Entity>();
    let player_health = health.get(*player);

    if let Some(player_health) = player_health {
        draw_health(player_health, 1, 1, 10, ctx);
    }

    ctx.draw_box(
        0,
        50,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let log = ecs.fetch::<super::gamelog::GameLog>();
    for (line, message) in log.entries.iter().rev().take(5).enumerate() {
        ctx.print(2, 50 + line + 1, message);
    }

    ctx.print(74, 1, format!("{} fps", ctx.fps));
    draw_tooltips(ecs, ctx);
}

fn draw_health(health: &Health, x: i32, y: i32, width: i32, ctx: &mut Rltk) {
    let frac_full = i32::max(width * health.current / health.max, 0);

    ctx.set_active_console(0);
    for i in 0..frac_full {
        ctx.set(
            x + i,
            y,
            rltk::RGB::named(rltk::RED),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('█'),
        );
    }
    for i in frac_full..width {
        ctx.set(
            x + i,
            y,
            rltk::RGB::named(rltk::DARKSALMON),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('█'),
        );
    }
    ctx.set_active_console(1);
    ctx.print(x, y, format!("{}/{}", health.current, health.max));
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
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
