use crate::{AttackIntent, RangeType};
use rltk::Point;

#[derive(Copy, Clone, PartialEq)]
pub enum AttackType {
    Sweep,
    Punch,
    Super,
}

// check if an attack is can be executed
// this returns the tile that will hit the target
pub fn is_attack_valid(
    attack_type: &AttackType,
    from_point: Point,
    target: Point,
) -> Option<Point> {
    let range_type = get_attack_range(attack_type);
    let shape = get_attack_shape(attack_type);

    for tile in crate::range_type::resolve_range_at(&range_type, from_point) {
        let affected_tiles = crate::range_type::resolve_range_at(&shape, tile);

        if affected_tiles.contains(&target) {
            return Some(tile);
        }
    }

    None
}

// convert an attack into an intent that can be executed by the event system
pub fn get_attack_intent(
    attack_type: &AttackType,
    loc: Point,
    attack_modifier: Option<AttackType>,
) -> AttackIntent {
    AttackIntent {
        main: *attack_type,
        modifier: attack_modifier,
        loc,
    }
}

fn get_intent_stat<T>(
    intent: &AttackIntent,
    retrieve: impl Fn(&AttackType) -> T,
    combine: impl FnOnce(T, T) -> T,
) -> T {
    let stat = retrieve(&intent.main);

    match intent.modifier {
        None => stat,
        Some(modifier) => {
            let modifier_stat = retrieve(&modifier);
            combine(stat, modifier_stat)
        }
    }
}

pub fn get_intent_name(intent: &AttackIntent) -> String {
    get_intent_stat(intent, get_attack_name, |x, y| format!("{} {}", y, x))
}

pub fn get_intent_power(intent: &AttackIntent) -> i32 {
    get_intent_stat(intent, get_attack_power, |x, y| x + y)
}

pub fn get_intent_speed(intent: &AttackIntent) -> i32 {
    get_intent_stat(intent, get_attack_speed, |x, y| x + y)
}

pub fn get_intent_guard(intent: &AttackIntent) -> i32 {
    get_intent_stat(intent, get_attack_guard, |x, y| x + y)
}

pub fn get_attack_range(attack_type: &AttackType) -> RangeType {
    match attack_type {
        AttackType::Sweep => RangeType::Single,
        AttackType::Punch => RangeType::Square { size: 1 },
        AttackType::Super => RangeType::Empty,
    }
}

pub fn get_attack_power(attack_type: &AttackType) -> i32 {
    match attack_type {
        AttackType::Sweep => 1,
        AttackType::Punch => 1,
        AttackType::Super => 2,
    }
}

pub fn get_attack_shape(attack_type: &AttackType) -> RangeType {
    match attack_type {
        AttackType::Sweep => RangeType::Square { size: 1 },
        AttackType::Punch => RangeType::Single,
        AttackType::Super => RangeType::Empty,
    }
}

pub fn get_attack_speed(attack_type: &AttackType) -> i32 {
    match attack_type {
        AttackType::Sweep => 0,
        AttackType::Punch => 1,
        AttackType::Super => -2,
    }
}

pub fn get_attack_guard(attack_type: &AttackType) -> i32 {
    match attack_type {
        AttackType::Sweep => 0,
        AttackType::Punch => 0,
        AttackType::Super => 0,
    }
}

pub fn get_attack_name(attack_type: &AttackType) -> String {
    let name = match attack_type {
        AttackType::Sweep => "sweep",
        AttackType::Punch => "punch",
        AttackType::Super => "super",
    };

    name.to_string()
}
