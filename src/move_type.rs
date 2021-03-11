use crate::{AttackIntent, RangeType};
use rltk::Point;

pub enum AttackType {
    Sweep,
    Punch,
}

// check if an attack is can be executed
// this checks for attack ranges as well as any extra conditions
pub fn is_attack_valid(attack_type: &AttackType, point1: Point, point2: Point) -> bool {
    let distance = rltk::DistanceAlg::Manhattan.distance2d(point1, point2);

    match attack_type {
        AttackType::Sweep => distance == 1.0,
        AttackType::Punch => distance == 1.0,
    }
}

// convert an attack into an intent that can be executed by the event system
pub fn get_attack_intent(attack_type: &AttackType, center: Point) -> AttackIntent {
    match attack_type {
        AttackType::Sweep => AttackIntent {
            name: "sweep".to_string(),
            loc: center,
            range: RangeType::Square { size: 1 },
        },
        AttackType::Punch => AttackIntent {
            name: "punch".to_string(),
            loc: center,
            range: RangeType::Single,
        },
    }
}

pub fn get_attack_name(attack_type: &AttackType) -> String {
    let name = match attack_type {
        AttackType::Sweep => "sweep",
        AttackType::Punch => "punch",
    };

    name.to_string()
}
