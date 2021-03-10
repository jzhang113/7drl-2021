use crate::{AttackIntent, RangeType};
use rltk::Point;

pub enum AttackType {
    Charge,
    Cleave,
}

pub fn is_attack_valid(attack_type: &AttackType, pos: Point) {}

pub fn get_attack_range(attack_type: &AttackType, pos1: Point, pos2: Point) -> AttackIntent {
    match attack_type {
        AttackType::Charge => AttackIntent {
            loc: pos2,
            range: RangeType::Custom {
                offsets: vec![(0, 1)],
            },
        },
        AttackType::Cleave => AttackIntent {
            loc: pos2,
            range: RangeType::Square { size: 1 },
        },
    }
}
