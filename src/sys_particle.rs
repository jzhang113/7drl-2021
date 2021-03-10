use super::{CardLifetime, ParticleLifetime, Position, Renderable};
use rltk::{FontCharType, Point, Rltk, RGB};
use specs::prelude::*;

pub fn cleanup_particles(ecs: &mut World, ctx: &Rltk) {
    let dead_particles = update_lifetimes(ecs, ctx);

    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Failed to delete particle");
    }
}

fn update_lifetimes(ecs: &mut World, ctx: &Rltk) -> Vec<Entity> {
    let mut dead_particles = Vec::new();
    let mut particles = ecs.write_storage::<ParticleLifetime>();
    let mut cards = ecs.write_storage::<CardLifetime>();
    let entities = ecs.entities();

    for (ent, mut lifetime) in (&entities, &mut particles).join() {
        lifetime.remaining -= ctx.frame_time_ms;
        if lifetime.remaining < 0.0 {
            dead_particles.push(ent);
        }
    }

    for (ent, mut lifetime) in (&entities, &mut cards).join() {
        lifetime.remaining -= ctx.frame_time_ms;
        if lifetime.remaining < 0.0 {
            dead_particles.push(ent);
        }
    }

    dead_particles
}

#[derive(Copy, Clone)]
pub struct ParticleRequest {
    pub position: Point,
    pub color: RGB,
    pub symbol: FontCharType,
    pub lifetime: f32,
}

pub struct CardRequest {
    pub name: String,
    pub offset: i32,
    pub affected: std::sync::Arc<Vec<rltk::Point>>,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
    card_stack: Vec<CardRequest>,
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {
            requests: Vec::new(),
            card_stack: Vec::new(),
        }
    }

    pub fn make_particle(&mut self, request: ParticleRequest) {
        self.requests.push(request);
    }

    pub fn make_card(&mut self, mut request: CardRequest, offset: i32) {
        request.offset = offset;
        self.card_stack.push(request);
    }
}

pub struct ParticleSpawnSystem;

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
        WriteStorage<'a, CardLifetime>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut lifetimes, mut cards, mut builder) =
            data;

        for request in builder.requests.drain(..) {
            let particle = entities.create();
            positions
                .insert(
                    particle,
                    Position {
                        x: request.position.x,
                        y: request.position.y,
                    },
                )
                .expect("Failed to insert Position for particle");
            renderables
                .insert(
                    particle,
                    Renderable {
                        symbol: request.symbol,
                        fg: request.color,
                        bg: rltk::RGB::named(rltk::BLACK),
                    },
                )
                .expect("Failed to insert Renderable for particle");
            lifetimes
                .insert(
                    particle,
                    ParticleLifetime {
                        base: request.lifetime,
                        remaining: request.lifetime,
                        should_fade: true,
                    },
                )
                .expect("Failed to insert ParticleLifetime for particle");
        }

        for (i, mut request) in builder.card_stack.drain(..).enumerate() {
            request.offset += i as i32;

            let card = entities.create();
            cards
                .insert(
                    card,
                    CardLifetime {
                        remaining: 400.0,
                        data: request,
                    },
                )
                .expect("Failed to insert CardLifetime for card");
        }
    }
}
