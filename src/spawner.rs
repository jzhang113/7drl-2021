use crate::*;
use rltk::{Point, RandomNumberGenerator, Rect};

pub struct Spawner<'a> {
    ecs: &'a mut World,
    blocked_tiles: &'a mut Vec<bool>,
    rng: &'a mut RandomNumberGenerator,
    map_width: i32,
}

impl<'a> Spawner<'a> {
    pub fn new(
        ecs: &'a mut World,
        blocked_tiles: &'a mut Vec<bool>,
        rng: &'a mut RandomNumberGenerator,
        map_width: i32,
    ) -> Self {
        Spawner {
            ecs,
            blocked_tiles,
            rng,
            map_width,
        }
    }

    pub fn build(
        &mut self,
        room: &Rect,
        min: i32,
        max: i32,
        builder: impl Fn(&mut World, Point) -> Entity,
    ) {
        let spawn_count = self.rng.range(min, max);

        for _ in 0..spawn_count {
            let dx = self.rng.range(1, room.width());
            let dy = self.rng.range(1, room.height());
            let xpos = room.x1 + dx;
            let ypos = room.y1 + dy;
            let index = ((ypos * self.map_width) + xpos) as usize;

            // don't spawn over something else
            if !self.blocked_tiles[index] {
                let _enemy = builder(self.ecs, rltk::Point::new(xpos, ypos));
                self.blocked_tiles[index] = true;
            }
        }
    }
}

// #region Player
pub fn build_player(ecs: &mut World, point: Point) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewable {
            name: "Player".to_string(),
            symbol: rltk::to_cp437('@'),
            list_index: None,
        })
        .with(Player)
        .with(Schedulable {
            current: 0,
            base: 24,
            delta: 4,
        })
        .with(Viewshed {
            visible: Vec::new(),
            dirty: true,
            range: 8,
        })
        .with(CanReactFlag)
        //.with(BlocksTile)
        .with(Health {
            current: 10,
            max: 10,
        })
        .build()
}
// #endregion

// #region Enemies
pub fn build_mook(ecs: &mut World, point: Point) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('x'),
            fg: RGB::named(rltk::LIGHT_BLUE),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewable {
            name: "Mook".to_string(),
            symbol: rltk::to_cp437('x'),
            list_index: None,
        })
        .with(Schedulable {
            current: 0,
            base: 24,
            delta: 4,
        })
        .with(Viewshed {
            visible: Vec::new(),
            dirty: true,
            range: 8,
        })
        .with(BlocksTile)
        .with(Health { current: 5, max: 5 })
        .with(Moveset {
            moves: vec![AttackType::Punch],
        })
        .build()
}
// #endregion

// #region Objects
pub fn build_barrel(ecs: &mut World, point: Point) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('#'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewable {
            name: "Barrel".to_string(),
            symbol: rltk::to_cp437('#'),
            list_index: None,
        })
        .with(BlocksTile)
        .with(Health { current: 2, max: 2 })
        .with(DeathTrigger {
            event: EventType::Damage {
                source_name: "explosion".to_string(),
                amount: 1,
            },
            range: RangeType::Square { size: 1 },
        })
        .build()
}
// #endregion
