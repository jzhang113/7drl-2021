use rltk::RGB;

pub fn card_priority_color() -> RGB {
    RGB::named(rltk::GREEN)
}

pub fn card_interrupted_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn card_blocked_color() -> RGB {
    RGB::named(rltk::ORANGE)
}

pub fn bg_color() -> RGB {
    RGB::named(rltk::BLACK)
}

pub fn attack_highlight_color() -> RGB {
    RGB::named(rltk::LIGHTBLUE)
}

pub fn hp_main_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn hp_alt_color() -> RGB {
    RGB::named(rltk::DARKRED)
}

pub fn card_select_color() -> RGB {
    RGB::named(rltk::GOLD)
}

pub fn select_highlight_color() -> RGB {
    RGB::named(rltk::GOLD)
}

pub fn text_highlight_color() -> RGB {
    RGB::named(rltk::GOLD)
}

pub fn text_inactive_color() -> RGB {
    RGB::named(rltk::GREY)
}

pub fn text_dead_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn map_floor_color() -> RGB {
    RGB::from_f32(0.0, 0.5, 0.5)
}

pub fn map_wall_color() -> RGB {
    RGB::from_f32(0., 1.0, 0.)
}

pub fn attack_source_color() -> RGB {
    RGB::named(rltk::LIGHTBLUE)
}

pub fn attack_target_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn slow_card_color() -> RGB {
    RGB::from_hex("#4E5166").unwrap()
}

pub fn fast_card_color() -> RGB {
    RGB::from_hex("#AFE0CE").unwrap()
}

pub fn valid_cursor_color() -> RGB {
    RGB::named(rltk::CYAN)
}

pub fn invalid_cursor_color() -> RGB {
    RGB::named(rltk::RED)
}

pub fn tiles_in_range_color() -> RGB {
    rltk::RGB::named(rltk::BLUE)
}

pub fn header_message_color() -> RGB {
    RGB::named(rltk::GOLD)
}

pub fn header_err_color() -> RGB {
    RGB::named(rltk::DARKGOLDENROD)
}

pub fn particle_hit_color() -> RGB {
    rltk::RGB::named(rltk::RED)
}
