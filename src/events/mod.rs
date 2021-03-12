use super::{AttackIntent, CardRequest};
use rltk::Point;
use specs::prelude::*;
use std::sync::{Arc, Mutex};

mod event_type;
pub mod range_type;

pub use event_type::EventType;
pub use range_type::*;

lazy_static! {
    static ref STACK: Mutex<Vec<Event>> = Mutex::new(Vec::new());
    pub static ref CARDSTACK: Mutex<Vec<CardRequest>> = Mutex::new(Vec::new());
}

struct Event {
    attack_intent: Option<AttackIntent>,
    resolver: Box<dyn event_type::EventResolver + Send>,
    name: Option<String>,
    source: Option<Entity>,
    target_tiles: Arc<Vec<Point>>,
    invokes_reaction: bool,
}

pub fn add_event(
    event_type: &EventType,
    source: Option<Entity>,
    range: &RangeType,
    loc: Point,
    invokes_reaction: bool,
) {
    let mut stack = STACK.lock().expect("Failed to lock STACK");
    let event = Event {
        attack_intent: None,
        resolver: event_type::get_resolver(event_type),
        name: event_type::get_name(event_type),
        source,
        target_tiles: Arc::new(range_type::resolve_range_at(range, loc)),
        invokes_reaction,
    };

    stack.push(event);
}

pub fn add_damage_event(intent: &AttackIntent, source: Option<Entity>, invokes_reaction: bool) {
    let mut stack = STACK.lock().expect("Failed to lock STACK");

    let damage_event = EventType::Damage {
        source_name: crate::move_type::get_intent_combined_name(intent),
        amount: crate::move_type::get_intent_combined_damage(intent),
    };
    let range = &crate::move_type::get_attack_shape(&intent.main);

    let event = Event {
        attack_intent: Some(*intent),
        resolver: event_type::get_resolver(&damage_event),
        name: Some(crate::move_type::get_intent_combined_name(intent)),
        source,
        target_tiles: Arc::new(range_type::resolve_range_at(range, intent.loc)),
        invokes_reaction,
    };

    stack.push(event);
}

pub fn process_stack(ecs: &mut World) {
    loop {
        let event = STACK.lock().expect("Failed to lock STACK").pop();
        match event {
            None => {
                break;
            }
            Some(event) => {
                if event.target_tiles.is_empty() {
                    // non-targetted events
                    process_event(ecs, event);
                } else {
                    let mut entities_hit = get_affected_entities(ecs, &event.target_tiles);

                    if let Some(card_name) = &event.name {
                        add_card_to_stack(
                            ecs,
                            &entities_hit,
                            card_name.clone(),
                            Arc::clone(&event.target_tiles),
                        );
                    }

                    entities_hit.retain(|ent| entity_can_react(ecs, event.source, ent));

                    // check if there are entities that can respond
                    if event.invokes_reaction && !entities_hit.is_empty() {
                        let mut can_act = ecs.write_storage::<super::CanActFlag>();

                        for entity in entities_hit {
                            can_act
                                .insert(entity, super::CanActFlag { is_reaction: true })
                                .expect("Failed to insert CanActFlag");
                        }

                        // put the event back on the stack and return control to the main loop
                        STACK.lock().expect("Failed to lock STACK").push(event);
                        break;
                    } else {
                        // otherwise resolve the event
                        process_event(ecs, event);
                    }
                }
            }
        }
    }
}

fn get_affected_entities(ecs: &mut World, targets: &Vec<Point>) -> Vec<Entity> {
    let mut affected = Vec::new();
    let positions = ecs.read_storage::<crate::Position>();
    let entities = ecs.entities();

    for (ent, pos) in (&entities, &positions).join() {
        for target in targets {
            if pos.as_point() == *target {
                affected.push(ent);
            }
        }
    }

    affected
}

fn entity_can_react(ecs: &mut World, source: Option<Entity>, target: &Entity) -> bool {
    let react_storage = ecs.read_storage::<super::CanReactFlag>();
    let can_react = react_storage.get(*target).is_some();

    match source {
        None => can_react,
        Some(source) => {
            if source == *target {
                false
            } else {
                can_react
            }
        }
    }
}

fn process_event(ecs: &mut World, event: Event) {
    let top_card = CARDSTACK.lock().expect("Failed to lock CARDSTACK").pop();
    let active_count = current_active_card_count(ecs);

    if let Some(top_card) = top_card {
        let mut builder = ecs.fetch_mut::<crate::ParticleBuilder>();
        builder.make_card(top_card, active_count);
    }

    // TODO: no clue if this can be simplified
    let stack_event = {
        // this lock needs to be limited in scope, since the resolver may want access to the stack via add_event
        let mut stack = STACK.lock().expect("Failed to lock STACK");
        stack.pop()
    };

    match stack_event {
        None => event
            .resolver
            .resolve(ecs, event.source, event.target_tiles.to_vec()),
        Some(stack_event) => match stack_event.attack_intent {
            None => event
                .resolver
                .resolve(ecs, event.source, event.target_tiles.to_vec()),
            Some(intent) => {
                println!("contested attack!");
            }
        },
    }
}

fn add_card_to_stack(
    ecs: &mut World,
    entities_hit: &Vec<Entity>,
    name: String,
    hit_range: Arc<Vec<rltk::Point>>,
) {
    let active_count = current_active_card_count(ecs);
    let player = ecs.fetch::<Entity>();

    if entities_hit.contains(&*player) {
        let visual_event_data = Some(CardRequest {
            name,
            offset: active_count,
            affected: hit_range,
        });

        if let Some(visual_event_data) = visual_event_data {
            CARDSTACK
                .lock()
                .expect("Failed to lock CARDSTACK")
                .push(visual_event_data);
        }
    }
}

fn current_active_card_count(ecs: &mut World) -> i32 {
    let cards = ecs.read_storage::<crate::CardLifetime>();
    cards.join().count() as i32
}
