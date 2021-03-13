use super::{AttackIntent, CardRequest};
use crate::move_type;
use rltk::Point;
use specs::prelude::*;
use std::sync::{Arc, Mutex};

mod event_type;
pub mod range_type;

pub use event_type::EventType;
pub use range_type::*;

const ATK_SPD_BONUS: i32 = 1;
const DEF_GUARD_BONUS: i32 = 1;

const SPEED_ROLL_RANGE: i32 = 6;
const GUARD_ROLL_RANGE: i32 = 6;

lazy_static! {
    static ref STACK: Mutex<Vec<Event>> = Mutex::new(Vec::new());
    static ref PROCESSING: Mutex<Option<Event>> = Mutex::new(None);
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

    let intent_name = move_type::get_intent_name(intent);
    let damage_event = EventType::Damage {
        source_name: intent_name.clone(),
        amount: move_type::get_intent_power(intent),
    };
    let range = &move_type::get_attack_shape(&intent.main);

    let event = Event {
        attack_intent: Some(*intent),
        resolver: event_type::get_resolver(&damage_event),
        name: Some(intent_name),
        source,
        target_tiles: Arc::new(range_type::resolve_range_at(range, intent.loc)),
        invokes_reaction,
    };

    stack.push(event);
}

pub fn process_stack(ecs: &mut World) -> crate::RunState {
    // if we have an event that was interrupted, resume it
    // also we need to wrap this mutex since we try to lock it again later
    let mut processing = PROCESSING.lock().expect("Failed to lock PROCESSING");
    let stashed_event = processing.take();

    *processing = None;

    if let Some(event) = stashed_event {
        // the stashed event is no longer in-progress
        if let Some(event_ent) = event.source {
            let mut in_progress = ecs.write_storage::<crate::AttackInProgress>();
            in_progress.remove(event_ent);
        }

        process_event(ecs, event);
    }

    loop {
        let event = STACK.lock().expect("Failed to lock STACK").pop();
        match event {
            None => return crate::RunState::Running,
            Some(event) => {
                if event.target_tiles.is_empty() {
                    // non-targetted events
                    process_event(ecs, event);
                } else {
                    let mut entities_hit = get_affected_entities(ecs, &event.target_tiles);

                    if let Some(intent) = &event.attack_intent {
                        add_card_to_stack(
                            ecs,
                            &entities_hit,
                            *intent,
                            event.source,
                            Arc::clone(&event.target_tiles),
                        );
                    }

                    entities_hit.retain(|ent| entity_can_react(ecs, event.source, ent));

                    // check if there are entities that can respond
                    if event.invokes_reaction && !entities_hit.is_empty() {
                        let mut can_act = ecs.write_storage::<super::CanActFlag>();
                        let mut scheds = ecs.write_storage::<super::Schedulable>();

                        for entity in entities_hit {
                            // if this entity is going to act, refund their time cost
                            if can_act.get(entity).is_some() {
                                let mut sched = scheds.get_mut(entity).unwrap();
                                sched.current -= sched.base;
                            }

                            can_act
                                .insert(
                                    entity,
                                    super::CanActFlag {
                                        is_reaction: true,
                                        reaction_target: event.source,
                                    },
                                )
                                .expect("Failed to insert CanActFlag");
                        }

                        // this event is now in-progress if it came from an entity
                        if let Some(event_ent) = event.source {
                            let mut in_progress = ecs.write_storage::<crate::AttackInProgress>();
                            in_progress
                                .insert(event_ent, crate::AttackInProgress)
                                .expect("couldn't mark event as in progress");
                        }
                        // stash the current event and return control to the main loop
                        *processing = Some(event);

                        return crate::RunState::AwaitingInput;
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

    match event.attack_intent {
        None => event
            .resolver
            .resolve(ecs, event.source, event.target_tiles.to_vec()),
        Some(event_intent) => {
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
                    None => {
                        // replace the stack event if we're not using it
                        {
                            STACK
                                .lock()
                                .expect("Failed to lock STACK")
                                .push(stack_event);
                        }

                        event
                            .resolver
                            .resolve(ecs, event.source, event.target_tiles.to_vec())
                    }
                    Some(stack_intent) => {
                        let (atk_speed_roll, def_speed_roll, def_guard_roll, atk_power_roll) = {
                            let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
                            let s1 = rng.range(0, SPEED_ROLL_RANGE);
                            let s2 = rng.range(0, SPEED_ROLL_RANGE);
                            let s3 = rng.range(0, GUARD_ROLL_RANGE);
                            let s4 = rng.range(0, GUARD_ROLL_RANGE);
                            (s1, s2, s3, s4)
                        };

                        {
                            let mut intents = ecs.fetch_mut::<crate::IntentData>();
                            intents.hidden = false;
                        }

                        // compare speed to determine which attack resolves first
                        let atk_speed = move_type::get_intent_speed(&event_intent)
                            + atk_speed_roll
                            + ATK_SPD_BONUS;
                        let def_speed = move_type::get_intent_speed(&stack_intent) + def_speed_roll;

                        let atk;
                        let atk_event;
                        let def;
                        let def_event;
                        let def_bonus_active;

                        if atk_speed > def_speed {
                            atk = event_intent;
                            atk_event = event;
                            def = stack_intent;
                            def_event = stack_event;
                            def_bonus_active = true;
                            println!("attacker wins the speed roll");
                        } else {
                            atk = stack_intent;
                            atk_event = stack_event;
                            def = event_intent;
                            def_event = event;
                            def_bonus_active = false;
                            println!("defender wins the speed roll");
                        }

                        atk_event.resolver.resolve(
                            ecs,
                            atk_event.source,
                            atk_event.target_tiles.to_vec(),
                        );

                        // compare power vs guard to determine if the defender can counter
                        // only the defender can gain the guard bonus
                        let mut def_guard = move_type::get_intent_guard(&def) + def_guard_roll;
                        if def_bonus_active {
                            def_guard += DEF_GUARD_BONUS;
                        }
                        let atk_power = move_type::get_intent_power(&atk) + atk_power_roll;

                        if atk_power > def_guard {
                            println!("defender is stunned!");
                            return;
                        }

                        def_event.resolver.resolve(
                            ecs,
                            def_event.source,
                            def_event.target_tiles.to_vec(),
                        );
                    }
                },
            }
        }
    }
}

fn add_card_to_stack(
    ecs: &mut World,
    entities_hit: &Vec<Entity>,
    intent: AttackIntent,
    source: Option<Entity>,
    hit_range: Arc<Vec<rltk::Point>>,
) {
    let active_count = current_active_card_count(ecs);
    let player = ecs.fetch::<Entity>();

    if entities_hit.contains(&*player) {
        let visual_event_data = Some(CardRequest {
            attack_intent: intent,
            source,
            offset: active_count,
            affected: hit_range,
        });

        if let Some(visual_event_data) = visual_event_data {
            CARDSTACK
                .lock()
                .expect("Failed to lock CARDSTACK")
                .push(visual_event_data);

            let mut intents = ecs.fetch_mut::<crate::IntentData>();

            intents.hidden = true;
            intents.prev_incoming_intent = Some(intent);
            intents.prev_outgoing_intent = None;
        }
    }
}

fn current_active_card_count(ecs: &mut World) -> i32 {
    let cards = ecs.read_storage::<crate::CardLifetime>();
    cards.join().count() as i32
}
