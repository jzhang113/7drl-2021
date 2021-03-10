use crate::ParticleRequest;
use rltk::Point;
use specs::prelude::*;

pub enum EventType {
    Damage { amount: i32 },
    ParticleSpawn { request: ParticleRequest },
    // ShowCard { request: CardRequest, offset: i32 },
}

pub fn get_name(event: &EventType) -> Option<String> {
    match event {
        EventType::Damage { .. } => Some("Damage".to_string()),
        _ => None,
    }
}

pub fn get_resolver(event: &EventType) -> Box<dyn EventResolver + Send> {
    match event {
        EventType::Damage { amount } => Box::new(DamageResolver { amount: *amount }),
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
                &crate::RangeType::Empty,
                Point::zero(),
                false,
            );
        }

        let affected = super::get_affected_entities(world, &targets);
        let mut healths = world.write_storage::<crate::Health>();

        for e_aff in affected.iter() {
            let affected = healths.get_mut(*e_aff);
            if let Some(mut affected) = affected {
                affected.current -= self.amount;
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
