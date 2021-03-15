use crate::ParticleRequest;
use rltk::{Algorithm2D, Point};
use specs::prelude::*;

const PARTICLE_HIT_LIFETIME: f32 = 600.0;

#[derive(PartialEq, Copy, Clone)]
pub enum DropType {
    Skill,
    Health,
}

#[derive(PartialEq, Copy, Clone)]
pub enum EventType {
    Damage { amount: i32 },
    Push { source_pos: Point, amount: i32 },
    Movement,
    ParticleSpawn { request: ParticleRequest },
    ItemDrop { drop_type: DropType, quality: i32 },
    // ShowCard { request: CardRequest, offset: i32 },
}

pub fn get_resolver(event: &EventType) -> Box<dyn EventResolver + Send> {
    match event {
        EventType::Damage { amount } => Box::new(DamageResolver { amount: *amount }),
        EventType::Push { source_pos, amount } => Box::new(PushResolver {
            source_pos: *source_pos,
            amount: *amount,
        }),
        EventType::Movement => Box::new(MovementResolver),
        EventType::ItemDrop { drop_type, quality } => Box::new(DropResolver {
            drop_type: *drop_type,
            quality: *quality,
        }),
        EventType::ParticleSpawn { request } => Box::new(ParticleResolver { request: *request }),
    }
}

pub trait EventResolver {
    fn resolve(&self, world: &mut World, source: Option<Entity>, targets: Vec<Point>) -> ();
}

pub struct DamageResolver {
    amount: i32,
}

impl EventResolver for DamageResolver {
    fn resolve(&self, world: &mut World, _source: Option<Entity>, targets: Vec<Point>) {
        for pos in targets.iter() {
            super::add_particle_event(*pos, crate::particle_hit_color(), PARTICLE_HIT_LIFETIME);
        }

        let affected = super::get_affected_entities(world, &targets);
        let mut healths = world.write_storage::<crate::Health>();
        let mut blocks = world.write_storage::<crate::BlockAttack>();

        for e_aff in affected.iter() {
            let mut damage_amount = self.amount;
            if let Some(block) = blocks.get(*e_aff) {
                damage_amount -= block.block_amount as i32;
                damage_amount = std::cmp::max(damage_amount, 0);

                blocks.remove(*e_aff);
            };

            let affected = healths.get_mut(*e_aff);
            if let Some(mut affected) = affected {
                affected.current -= damage_amount;
            }
        }
    }
}

pub struct ParticleResolver {
    request: ParticleRequest,
}

impl EventResolver for ParticleResolver {
    fn resolve(&self, world: &mut World, _source: Option<Entity>, _targets: Vec<Point>) {
        let mut builder = world.fetch_mut::<crate::ParticleBuilder>();
        builder.make_particle(self.request);
    }
}

pub struct PushResolver {
    source_pos: Point,
    amount: i32,
}

impl EventResolver for PushResolver {
    fn resolve(&self, world: &mut World, _source: Option<Entity>, targets: Vec<Point>) {
        for pos in targets.iter() {
            super::add_particle_event(*pos, crate::particle_hit_color(), PARTICLE_HIT_LIFETIME);
        }

        let affected = super::get_affected_entities(world, &targets);
        let mut positions = world.write_storage::<crate::Position>();
        let mut map = world.fetch_mut::<crate::Map>();

        for e_aff in affected.iter() {
            let affected = positions.get_mut(*e_aff);
            if let Some(mut affected) = affected {
                // find the closest direction to push
                let dx = i32::signum(affected.x - self.source_pos.x);
                let dy = i32::signum(affected.y - self.source_pos.y);

                let mut next_x = affected.x;
                let mut next_y = affected.y;

                // push along the direction until we hit something else
                for _ in 0..self.amount {
                    let possible = Point::new(next_x + dx, next_y + dy);

                    if !map.in_bounds(possible) {
                        break;
                    }

                    let possible_index = map.point2d_to_index(possible);
                    if map.blocked_tiles[possible_index] {
                        break;
                    }

                    next_x = possible.x;
                    next_y = possible.y;
                }

                // fix indexing
                let affected_index = map.get_index(affected.x, affected.y);
                let next_index = map.get_index(next_x, next_y);
                map.blocked_tiles[affected_index] = false;
                map.blocked_tiles[next_index] = true;

                affected.x = next_x;
                affected.y = next_y;
            }
        }
    }
}

pub struct MovementResolver;

impl EventResolver for MovementResolver {
    fn resolve(&self, world: &mut World, source: Option<Entity>, targets: Vec<Point>) {
        if let Some(source) = source {
            let mut positions = world.write_storage::<crate::Position>();
            let mut map = world.fetch_mut::<crate::Map>();

            if let Some(source_pos) = positions.get_mut(source) {
                // if we have more than one target position to move to, pick at random
                if targets.len() > 0 {
                    let target = targets[0];

                    // confirm target is valid
                    let target_index = map.point2d_to_index(target);
                    if !map.in_bounds(target) || map.blocked_tiles[target_index] {
                        return;
                    }

                    // fix indexing
                    let source_index = map.get_index(source_pos.x, source_pos.y);
                    map.blocked_tiles[source_index] = false;
                    map.blocked_tiles[target_index] = true;

                    source_pos.x = target.x;
                    source_pos.y = target.y;
                }
            }
        }
    }
}

pub struct DropResolver {
    drop_type: DropType,
    quality: i32,
}

impl EventResolver for DropResolver {
    fn resolve(&self, world: &mut World, _source: Option<Entity>, targets: Vec<Point>) {
        // get a random open square
        if targets.len() == 0 {
            return;
        }

        // TODO: this is fine for now, we are only using single targets anyways
        let drop_point = targets[0];

        match self.drop_type {
            DropType::Health => {
                let heal_amount = {
                    let mut rng = world.fetch_mut::<rltk::RandomNumberGenerator>();
                    let heal_amount = rng.range(self.quality / 2, self.quality / 2 + 2);
                    std::cmp::max(heal_amount, 1)
                };

                let heal_item = world
                    .create_entity()
                    .with(crate::Position {
                        x: drop_point.x,
                        y: drop_point.y,
                    })
                    .with(crate::Renderable {
                        symbol: rltk::to_cp437('+'),
                        fg: crate::health_color(),
                        bg: crate::bg_color(),
                    })
                    .with(crate::Heal {
                        amount: heal_amount as u32,
                    })
                    .with(crate::Viewable {
                        name: "health".to_string(),
                        symbol: rltk::to_cp437('+'),
                        description: vec!["Packaged health, don't ask".to_string()],
                        list_index: None,
                    })
                    .build();

                let mut map = world.fetch_mut::<crate::Map>();
                map.track_item(heal_item, drop_point);
            }
            DropType::Skill => {
                let skill_choices = {
                    let mut rng = world.fetch_mut::<rltk::RandomNumberGenerator>();
                    let mut skill_ary = Vec::new();

                    // generate 3 choices by default
                    for _ in 0..3 {
                        skill_ary.push(crate::deck::attack_type_table(&mut rng, self.quality));
                    }

                    skill_ary
                };

                let book_item = world
                    .create_entity()
                    .with(crate::Position {
                        x: drop_point.x,
                        y: drop_point.y,
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
                        symbol: rltk::to_cp437('?'),
                        description: vec!["An old fighting manual".to_string()],
                        list_index: None,
                    })
                    .build();

                let mut map = world.fetch_mut::<crate::Map>();
                map.track_item(book_item, drop_point);
            }
        }
    }
}
