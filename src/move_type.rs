use crate::{AttackIntent, RangeType};
use rltk::Point;

#[derive(Copy, Clone, PartialEq)]
pub enum AttackType {
    Sweep,
    Punch,
}

// check if an attack is can be executed
// this checks for attack ranges as well as any extra conditions
pub fn is_attack_valid(attack_type: &AttackType, point1: Point, point2: Point) -> bool {
    let distance = rltk::DistanceAlg::Manhattan.distance2d(point1, point2);
    let max_range = get_attack_range(attack_type) as f32;

    distance <= max_range
}

// convert an attack into an intent that can be executed by the event system
pub fn get_attack_intent(attack_type: &AttackType, loc: Point) -> AttackIntent {
    let name = get_attack_name(attack_type);
    let range = get_attack_shape(attack_type);

    AttackIntent { name, loc, range }
}

pub fn get_attack_range(attack_type: &AttackType) -> i32 {
    match attack_type {
        AttackType::Sweep => 1,
        AttackType::Punch => 1,
    }
}

pub fn get_attack_shape(attack_type: &AttackType) -> RangeType {
    match attack_type {
        AttackType::Sweep => RangeType::Square { size: 1 },
        AttackType::Punch => RangeType::Single,
    }
}

pub fn get_attack_name(attack_type: &AttackType) -> String {
    let name = match attack_type {
        AttackType::Sweep => "sweep",
        AttackType::Punch => "punch",
    };

    name.to_string()
}
