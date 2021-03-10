use rltk::{Point, RGB};
use specs::prelude::*;
use specs::Component;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn as_point(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

#[derive(Component)]
pub struct Renderable {
    pub symbol: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Viewshed {
    pub visible: Vec<Point>,
    pub dirty: bool,
    pub range: i32,
}

#[derive(Component)]
pub struct CanActFlag {
    pub is_reaction: bool,
}

#[derive(Component)]
pub struct CanReactFlag;

#[derive(Component)]
pub struct Schedulable {
    pub current: i32,
    pub base: i32,
    pub delta: i32,
}

#[derive(Component)]
pub struct ParticleLifetime {
    pub base: f32,
    pub remaining: f32,
    pub should_fade: bool,
}

#[derive(Component)]
pub struct CardLifetime {
    pub remaining: f32,
    pub data: super::CardRequest,
}

#[derive(Component)]
pub struct BlocksTile;

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct DeathTrigger {
    pub event: crate::EventType,
    pub range: crate::RangeType,
}

#[derive(Component)]
pub struct AttackIntent {
    pub loc: rltk::Point,
    pub range: crate::RangeType,
}

#[derive(Component)]
pub struct MoveIntent {
    pub loc: rltk::Point,
}

#[derive(Component)]
pub struct Moveset {
    pub moves: Vec<crate::AttackType>,
}
