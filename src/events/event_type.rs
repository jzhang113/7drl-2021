use crate::ParticleRequest;
use rltk::{Algorithm2D, Point};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum EventType {
    Damage { amount: i32 },
    Push { source_pos: Point, amount: i32 },
    ParticleSpawn { request: ParticleRequest },
    // ShowCard { request: CardRequest, offset: i32 },
}

pub fn get_resolver(event: &EventType) -> Box<dyn EventResolver + Send> {
    match event {
        EventType::Damage { amount } => Box::new(DamageResolver { amount: *amount }),
        EventType::Push { source_pos, amount } => Box::new(PushResolver {
            source_pos: *source_pos,
            amount: *amount,
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
            super::add_event(
                &EventType::ParticleSpawn {
                    request: ParticleRequest {
                        position: *pos,
                        color: rltk::RGB::named(rltk::RED),
                        symbol: rltk::to_cp437('█'),
                        lifetime: 600.0,
                    },
                },
                None,
                &crate::RangeType::Empty,
                Point::zero(),
                false,
            );
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
            super::add_event(
                &EventType::ParticleSpawn {
                    request: ParticleRequest {
                        position: *pos,
                        color: rltk::RGB::named(rltk::RED),
                        symbol: rltk::to_cp437('█'),
                        lifetime: 600.0,
                    },
                },
                None,
                &crate::RangeType::Empty,
                Point::zero(),
                false,
            );
        }

        let affected = super::get_affected_entities(world, &targets);
        let mut positions = world.write_storage::<crate::Position>();
        let map = world.fetch::<crate::Map>();

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

                affected.x = next_x;
                affected.y = next_y;
            }
        }
    }
}
