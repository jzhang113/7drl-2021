use crate::*;
use rltk::{Point, RandomNumberGenerator, Rect};

pub struct Spawner<'a> {
    ecs: &'a mut World,
    map: &'a mut Map,
    map_width: i32,
}

impl<'a> Spawner<'a> {
    pub fn new(ecs: &'a mut World, map: &'a mut Map, map_width: i32) -> Self {
        Spawner {
            ecs,
            map,
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
        let mut spawn_points = Vec::new();
        {
            let mut rng = self.ecs.fetch_mut::<RandomNumberGenerator>();
            let spawn_count = rng.range(min, max);

            for _ in 0..spawn_count {
                let dx = rng.range(1, room.width());
                let dy = rng.range(1, room.height());
                let xpos = room.x1 + dx;
                let ypos = room.y1 + dy;
                let index = ((ypos * self.map_width) + xpos) as usize;

                // don't spawn over something else
                if !self.map.blocked_tiles[index] && index != self.map.level_exit {
                    spawn_points.push((xpos, ypos));
                }
            }
        }

        for (xpos, ypos) in spawn_points {
            let point = Point::new(xpos, ypos);
            let enemy = builder(self.ecs, point);
            self.map.track_creature(enemy, point);
        }
    }

    pub fn build_choice(
        &mut self,
        room: &Rect,
        min: i32,
        max: i32,
        quality: i32,
        chance: Vec<f32>,
        builder: Vec<impl Fn(&mut World, Point, i32) -> Entity>,
    ) {
        let mut spawn_points = Vec::new();
        {
            let mut rng = self.ecs.fetch_mut::<RandomNumberGenerator>();
            let spawn_count = rng.range(min, max);

            for _ in 0..spawn_count {
                let dx = rng.range(1, room.width());
                let dy = rng.range(1, room.height());
                let xpos = room.x1 + dx;
                let ypos = room.y1 + dy;
                let index = ((ypos * self.map_width) + xpos) as usize;

                // don't spawn over something else
                if !self.map.blocked_tiles[index] && index != self.map.level_exit {
                    let roll = rng.rand::<f32>();
                    let mut cumul_prob = 0.0;
                    let mut builder_index = 0;

                    for index in 0..chance.len() {
                        cumul_prob += chance[index];

                        if roll < cumul_prob {
                            builder_index = index;
                            break;
                        }
                    }

                    spawn_points.push((builder_index, xpos, ypos));
                }
            }
        }

        for (builder_index, xpos, ypos) in spawn_points {
            let point = Point::new(xpos, ypos);
            let enemy = builder[builder_index](self.ecs, point, quality);
            self.map.track_creature(enemy, point);
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
            description: vec!["That's you!".to_string()],
            seen: false,
        })
        .with(ViewableIndex { list_index: None })
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
            description: vec![
                "A lowly grunt,".to_string(),
                "unskilled, but".to_string(),
                "can still pack".to_string(),
                "a wallop".to_string(),
            ],
            seen: false,
        })
        .with(ViewableIndex { list_index: None })
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
            moves: vec![(AttackType::Haymaker, 0.25), (AttackType::Punch, 0.75)],
        })
        .with(AiState {
            status: Behavior::Wander,
            tracking: None,
        })
        .build()
}
// #endregion

// #region Objects
fn barrel_builder(ecs: &mut World, point: Point) -> EntityBuilder {
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
            description: vec![
                "A barrel, what".to_string(),
                "could be".to_string(),
                "inside?".to_string(),
            ],
            seen: false,
        })
        .with(BlocksTile)
        .with(Health { current: 2, max: 2 })
}

pub fn build_empty_barrel(ecs: &mut World, point: Point, _quality: i32) -> Entity {
    barrel_builder(ecs, point).build()
}

pub fn build_exploding_barrel(ecs: &mut World, point: Point, quality: i32) -> Entity {
    barrel_builder(ecs, point)
        .with(DeathTrigger {
            event: EventType::Damage {
                amount: 1 + quality / 2,
            },
            range: RangeType::Square {
                size: 1 + quality / 3,
            },
        })
        .build()
}

pub fn build_health_barrel(ecs: &mut World, point: Point, quality: i32) -> Entity {
    barrel_builder(ecs, point)
        .with(DeathTrigger {
            event: EventType::ItemDrop {
                drop_type: crate::events::DropType::Health,
                quality,
            },
            range: RangeType::Single,
        })
        .build()
}

pub fn build_book_barrel(ecs: &mut World, point: Point, quality: i32) -> Entity {
    barrel_builder(ecs, point)
        .with(DeathTrigger {
            event: EventType::ItemDrop {
                drop_type: crate::events::DropType::Skill,
                quality,
            },
            range: RangeType::Single,
        })
        .build()
}

pub fn build_health_pickup(ecs: &mut World, point: Point, quality: i32) -> Entity {
    ecs.create_entity()
        .with(crate::Position {
            x: point.x,
            y: point.y,
        })
        .with(crate::Renderable {
            symbol: rltk::to_cp437('+'),
            fg: crate::health_color(),
            bg: crate::bg_color(),
        })
        .with(crate::Heal {
            amount: quality as u32,
        })
        .with(crate::Viewable {
            name: "health".to_string(),
            description: vec!["Packaged health, don't ask".to_string()],
            seen: false,
        })
        .build()
}

pub fn build_skill_pickup(
    ecs: &mut World,
    point: Point,
    skill_choices: Vec<crate::AttackType>,
) -> Entity {
    ecs.create_entity()
        .with(crate::Position {
            x: point.x,
            y: point.y,
        })
        .with(crate::Renderable {
            symbol: rltk::to_cp437('?'),
            fg: crate::health_color(),
            bg: crate::bg_color(),
        })
        .with(crate::SkillChoice {
            choices: skill_choices,
        })
        .with(crate::Viewable {
            name: "book".to_string(),
            description: vec!["An old fighting manual".to_string()],
            seen: false,
        })
        .build()
}
// #endregion
