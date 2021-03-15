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
            let mut fg = map.color_map[idx];

            match tile {
                TileType::Floor => {
                    symbol = rltk::to_cp437('.');
                }
                TileType::Wall => {
                    symbol = rltk::to_cp437('#');
                }
            }

            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }
            ctx.set(MAP_X + x, MAP_Y + y, fg, bg_color(), symbol);
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
                let base_color = bg_color();

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

pub fn draw_active_attacks(ecs: &World, ctx: &mut Rltk) {
    let card_stack_active = crate::events::CARDSTACK
        .lock()
        .expect("Failed to lock CARDSTACK");

    for card in card_stack_active.iter() {
        if let Some(attack_ent) = card.source {
            // highlight source on map
            let positions = ecs.read_storage::<Position>();
            let attack_pos = positions
                .get(attack_ent)
                .expect("Unexpected attack source without a position");

            highlight_bg(ctx, &Position::as_point(attack_pos), attack_source_color());
        }

        // highlight affected tiles
        for pos in card.affected.iter() {
            highlight_bg(ctx, pos, attack_target_color());
        }
    }
}

fn highlight_bg(ctx: &mut Rltk, pos: &rltk::Point, color: RGB) {
    ctx.set_active_console(0);
    ctx.set_bg(MAP_X + pos.x, MAP_Y + pos.y, color);
    ctx.set_active_console(1);
}

pub fn draw_intents(ecs: &World, ctx: &mut Rltk) {
    let intents = ecs.fetch::<IntentData>();

    ctx.print(4, 13, "INCOMING");
    if let Some(incoming) = intents.prev_incoming_intent {
        if intents.hidden {
            draw_card_hidden(ctx, 3, 14, RGB::named(rltk::WHITE));
        } else {
            draw_card_combined(ctx, incoming, 3, 14, &*intents, true);
        }
    }

    ctx.print(4, 32, "OUTGOING");
    if let Some(outgoing) = intents.prev_outgoing_intent {
        draw_card_combined(ctx, outgoing, 3, 33, &*intents, false);
    }
}

fn format_roll_bonuses<'a>(
    rolls: IntentRolls,
    speed_check: bool,
    incoming: bool,
) -> (String, String, String) {
    let (s1, s2, s3, s4) = rolls;

    let incoming_strings = {
        let power_str = format!("+{}", s4);
        let speed_str = format!("+{}", s1);
        let guard_str = format!(" 0");
        (power_str, speed_str, guard_str)
    };

    let outgoing_strings = {
        let power_str = format!(" 0");
        let speed_str = format!("+{}", s2);
        let guard_str = format!("+{}", s3);
        (power_str, speed_str, guard_str)
    };

    if speed_check {
        if incoming {
            incoming_strings
        } else {
            outgoing_strings
        }
    } else {
        if incoming {
            outgoing_strings
        } else {
            incoming_strings
        }
    }
}

fn draw_card_hidden(ctx: &mut Rltk, x_start: i32, y_start: i32, fore_color: RGB) {
    ctx.draw_box(x_start, y_start, CARD_W, CARD_H, fore_color, bg_color());

    ctx.print(x_start + 1, y_start + 1, "???");
}

fn draw_card_hand(
    ctx: &mut Rltk,
    attack: &AttackType,
    x_start: i32,
    y_start: i32,
    index: i32,
    selected: bool,
) {
    let timing = crate::move_type::get_attack_timing(&attack);
    let border_color = if selected {
        card_select_color()
    } else {
        match timing {
            crate::AttackTiming::Slow => slow_card_color(),
            crate::AttackTiming::Fast => fast_card_color(),
        }
    };

    ctx.draw_box(x_start, y_start, CARD_W, CARD_H, border_color, bg_color());

    let name = format!(
        "{}) {}",
        index + 1,
        crate::move_type::get_attack_name(attack)
    );
    ctx.print(x_start + 1, y_start + 1, name);

    let power_str = format!("{}", move_type::get_attack_power(attack));
    let speed_str = format!("{}", move_type::get_attack_speed(attack));
    let guard_str = format!("{}", move_type::get_attack_guard(attack));
    draw_card_stats(
        ctx,
        x_start,
        y_start + 3,
        &power_str,
        &speed_str,
        &guard_str,
    );

    let y_timing = y_start + 5;
    let timing_str = match timing {
        crate::AttackTiming::Slow => "SLOW",
        crate::AttackTiming::Fast => "FAST",
    };

    ctx.print(x_start + 1, y_timing, timing_str);
    let mut y_trait_line = y_start + 7;
    let y_type_line = y_start + CARD_H - 1;

    for att_trait in move_type::get_attack_traits(attack) {
        match att_trait {
            crate::AttackTrait::Damage => ctx.print(x_start + 1, y_type_line, "Attack "),
            crate::AttackTrait::Modifier => ctx.print(x_start + 1, y_type_line, "Modifier"),
            crate::AttackTrait::Movement => ctx.print(x_start + 1, y_type_line, "Movement"),
            crate::AttackTrait::Equipment => ctx.print(x_start + 1, y_type_line, "Equipment"),
            crate::AttackTrait::Knockback { amount } => {
                ctx.print(x_start + 1, y_trait_line, format!("PUSH {}", amount - 1));
                y_trait_line += 2;
            }
            crate::AttackTrait::Draw { amount } => {
                ctx.print(x_start + 1, y_trait_line, format!("DRAW {}", amount));
                y_trait_line += 2;
            }
        }
    }
}

fn draw_card_combined(
    ctx: &mut Rltk,
    attack: AttackIntent,
    x_start: i32,
    y_start: i32,
    intents: &IntentData,
    incoming: bool,
) {
    let mut border_color = RGB::named(rltk::WHITE);
    if intents.incoming_went_first && incoming {
        border_color = card_priority_color();
    } else if !intents.incoming_went_first && !incoming {
        border_color = card_priority_color();
    }

    if intents.defender_was_interrupted {
        if intents.incoming_went_first && !incoming {
            border_color = card_interrupted_color();
        } else if !intents.incoming_went_first && incoming {
            border_color = card_interrupted_color();
        }
    }

    if intents.prev_incoming_intent.is_none() {
        border_color = RGB::named(rltk::WHITE);
    }

    if intents.prev_outgoing_intent.is_none() {
        border_color = card_blocked_color();
    }

    ctx.draw_box(x_start, y_start, CARD_W, CARD_H, border_color, bg_color());

    let name = move_type::get_intent_name(&attack);
    ctx.print(x_start + 1, y_start + 1, name);

    let power_str = format!("{}", move_type::get_intent_power(&attack));
    let speed_str = format!("{}", move_type::get_intent_speed(&attack));
    let guard_str = format!("{}", move_type::get_intent_guard(&attack));
    draw_card_stats(
        ctx,
        x_start,
        y_start + 3,
        &power_str,
        &speed_str,
        &guard_str,
    );

    let y_stats = y_start + 5;
    let (power_str, speed_str, guard_str) =
        format_roll_bonuses(intents.rolls, intents.incoming_went_first, incoming);
    ctx.print(x_start + 3 - (power_str.len() as i32), y_stats, power_str);
    ctx.print(x_start + 6 - (speed_str.len() as i32), y_stats, speed_str);
    ctx.print(x_start + 9 - (guard_str.len() as i32), y_stats, guard_str);
}

fn draw_card_stats(
    ctx: &mut Rltk,
    x: i32,
    y: i32,
    power_str: &str,
    speed_str: &str,
    guard_str: &str,
) {
    ctx.print(x + 3 - (power_str.len() as i32), y, power_str);
    ctx.print(x + 6 - (speed_str.len() as i32), y, speed_str);
    ctx.print(x + 9 - (guard_str.len() as i32), y, guard_str);

    // stat icons
    ctx.set_active_console(2);
    ctx.set(x + 3, y, RGB::named(rltk::RED), RGB::named(rltk::BLACK), 1);
    ctx.set(
        x + 6,
        y,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        2,
    );
    ctx.set(
        x + 9,
        y,
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
        let xpos = start_x + (CARD_W + 1) * (i as i32);
        let ypos = CARD_Y;
        let index = i as i32;
        draw_card_hand(ctx, card, xpos, ypos, index, index == deck.selected);
    }
}

fn pluralize(root: String, count: i32) -> String {
    if count != 1 {
        return format!("{} {}s", count, root);
    } else {
        return format!("1 {}", root);
    }
}

pub fn draw_sidebar(ecs: &World, ctx: &mut Rltk) {
    let healths = ecs.read_storage::<Health>();
    let mut viewables = ecs.write_storage::<Viewable>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let positions = ecs.read_storage::<Position>();
    let in_progress = ecs.read_storage::<AttackInProgress>();

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

    for (mut view, pos, health, attack) in
        (&mut viewables, &positions, &healths, (&in_progress).maybe()).join()
    {
        if !player_view
            .visible
            .iter()
            .any(|view_pos| view_pos.x == pos.x && view_pos.y == pos.y)
        {
            continue;
        }

        view.list_index = Some(index);

        if index <= 5 {
            // change symbol color if attacking
            let symbol_color;
            if attack.is_some() {
                symbol_color = attack_highlight_color();
            } else {
                symbol_color = RGB::named(rltk::WHITE);
            }

            ctx.set(x, y, symbol_color, RGB::named(rltk::BLACK), view.symbol);
            ctx.set(
                x + 1,
                y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                rltk::to_cp437(':'),
            );

            let curr_hp = std::cmp::max(0, health.current);

            for i in 0..curr_hp {
                ctx.set(
                    x + i + 2,
                    y,
                    hp_main_color(),
                    bg_color(),
                    rltk::to_cp437('o'),
                );
            }

            for i in curr_hp..health.max {
                ctx.set(
                    x + i + 2,
                    y,
                    hp_alt_color(),
                    bg_color(),
                    rltk::to_cp437('o'),
                );
            }
        }

        y += 2;
        index += 1;

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

    // don't clear the previous line in hitpause
    match *status {
        RunState::HitPause { .. } => {}
        _ => ctx.cls(),
    };

    let x = 0;
    let y = CONSOLE_HEIGHT - 1;
    let icon_color = text_highlight_color();
    let bg_color = bg_color();
    let inactive_color = text_inactive_color();

    let is_reaction = {
        let can_act = ecs.read_storage::<super::CanActFlag>();
        let player = ecs.fetch::<Entity>();
        can_act
            .get(*player)
            .expect("uh-oh, we're waiting for input but the player can't act")
            .is_reaction
    };

    if is_reaction {
        ctx.print(CONSOLE_WIDTH - 6, y, "REACT");
    } else {
        ctx.print_color(CONSOLE_WIDTH - 5, y, inactive_color, bg_color, "MAIN");
    }

    match *status {
        RunState::AwaitingInput => {
            // movement controls
            if is_reaction {
                draw_movement_controls(ctx, x, y, inactive_color, bg_color, true);
            } else {
                draw_movement_controls(ctx, x, y, icon_color, bg_color, false);
            }

            // examine
            let view_section_x = 13;
            ctx.print_color(view_section_x, y, icon_color, bg_color, "v");
            ctx.print(view_section_x + 1, y, "iew map");

            // space bar
            let space_section_x = 25;
            let space_action_str;
            if is_reaction {
                space_action_str = "block";
            } else {
                space_action_str = "draw card";
            }

            ctx.print_color(space_section_x, y, icon_color, bg_color, "[SPACE]");
            ctx.print(space_section_x + 8, y, space_action_str);

            // card section
            let card_section_x = 45;
            ctx.print_color(card_section_x, y, icon_color, bg_color, "[1-7]");
            ctx.print(card_section_x + 6, y, "use card");
        }
        RunState::Targetting {
            attack_type: _,
            ignore_targetting,
        } => {
            // movement controls
            if ignore_targetting {
                draw_movement_controls(ctx, x, y, inactive_color, bg_color, true);
            } else {
                draw_movement_controls(ctx, x, y, icon_color, bg_color, false);
            }

            // examine
            let view_section_x = 13;
            ctx.print_color(view_section_x, y, icon_color, bg_color, "v");
            ctx.print(view_section_x + 1, y, "iew card");

            // space bar
            let space_section_x = 25;
            ctx.print_color(space_section_x, y, icon_color, bg_color, "[SPACE]");
            ctx.print(space_section_x + 8, y, "confirm");

            // escape
            let escape_section_x = 45;
            ctx.print_color(escape_section_x, y, icon_color, bg_color, "[ESC]");
            ctx.print(escape_section_x + 6, y, "cancel");

            // tab target
            let tab_section_x = 60;
            if ignore_targetting {
                ctx.print_color(tab_section_x, y, inactive_color, bg_color, "[TAB]");
                ctx.print_color(
                    tab_section_x + 6,
                    y,
                    inactive_color,
                    bg_color,
                    "next target",
                );
            } else {
                ctx.print_color(tab_section_x, y, icon_color, bg_color, "[TAB]");
                ctx.print(tab_section_x + 6, y, "next target");
            }
        }
        RunState::ViewEnemy { .. } => {
            // movement controls
            draw_movement_controls(ctx, x, y, icon_color, bg_color, false);

            // escape
            let escape_section_x = 13;
            ctx.print_color(escape_section_x, y, icon_color, bg_color, "[ESC]");
            ctx.print(escape_section_x + 6, y, "cancel");
        }
        RunState::Dead => {
            // restart
            ctx.print_color(x, y, icon_color, bg_color, "r");
            ctx.print(x + 1, y, "estart");
            ctx.print_color(CONSOLE_WIDTH - 6, y, text_dead_color(), bg_color, " DEAD");
        }
        RunState::HitPause { .. } => {
            ctx.print_color(CONSOLE_WIDTH - 6, y, inactive_color, bg_color, " WAIT");
        }
        _ => {}
    }

    ctx.set_active_console(1);
}

fn draw_movement_controls(ctx: &mut Rltk, x: i32, y: i32, fg: RGB, bg: RGB, inactive: bool) {
    ctx.set(x + 1, y, fg, bg, 27);
    ctx.set(x + 2, y, fg, bg, 25);
    ctx.set(x + 3, y, fg, bg, 24);
    ctx.set(x + 4, y, fg, bg, 26);

    if inactive {
        ctx.print_color(x + 6, y, fg, bg, "move");
    } else {
        ctx.print(x + 6, y, "move");
    }
}

pub fn draw_viewable_info(ecs: &World, ctx: &mut Rltk, entity: &Entity, index: u32) {
    let selected_color = select_highlight_color();
    let bg_color = bg_color();

    ctx.set(
        0,
        2 * index + 1,
        text_highlight_color(),
        bg_color,
        rltk::to_cp437('>'),
    );

    let positions = ecs.read_storage::<Position>();
    let viewables = ecs.read_storage::<Viewable>();
    let healths = ecs.read_storage::<Health>();
    let atk_in_progress = ecs.read_storage::<AttackInProgress>();
    let blocking = ecs.read_storage::<BlockAttack>();

    let pos = positions
        .get(*entity)
        .expect("viewable didn't have a position");
    let view = viewables.get(*entity).expect("viewable didn't have a view");
    let health = healths.get(*entity).expect("viewable didn't have health");

    let x = MAP_X + pos.x;
    let y = MAP_Y + pos.y;

    highlight_bg(ctx, &Position::as_point(pos), selected_color);

    let (box_x, box_y) = position_box(ctx, x, y, 15, 10, selected_color, bg_color);

    ctx.print(box_x + 1, box_y, view.name.clone());
    ctx.print(
        box_x + 1,
        box_y + 1,
        format!("HP: {}/{}", health.current, health.max),
    );

    if atk_in_progress.get(*entity).is_some() {
        ctx.print(box_x + 1, box_y + 3, "Attacking");
    } else if blocking.get(*entity).is_some() {
        ctx.print(box_x + 1, box_y + 3, "Blocking");
    } else {
        ctx.print(box_x + 1, box_y + 3, "Idle");
    }

    for (i, line) in view.description.iter().enumerate() {
        ctx.print(box_x + 1, box_y + 5 + i as i32, line.clone());
    }
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
