use crate::ParticleRequest;
use rltk::{Algorithm2D, Point};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum EventType {
    Damage { amount: i32 },
    Push { source_pos: Point, amount: i32 },
    Movement,
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
        EventType::Movement => Box::new(MovementResolver),
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
            super::add_particle_event(*pos, rltk::RGB::named(rltk::RED), 600.0);
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
            super::add_particle_event(*pos, rltk::RGB::named(rltk::RED), 600.0);
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
