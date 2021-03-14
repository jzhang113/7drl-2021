use super::{AttackIntent, CardRequest};
use crate::move_type;
use rltk::Point;
use specs::prelude::*;
use std::sync::{Arc, Mutex};

mod event_type;
pub mod range_type;

pub use event_type::EventType;
pub use range_type::*;

const ATK_SPD_BONUS: i32 = 0;
const DEF_GUARD_BONUS: i32 = 1;

const SPEED_ROLL_RANGE: i32 = 6;
const GUARD_ROLL_RANGE: i32 = 6;

lazy_static! {
    static ref STACK: Mutex<Vec<Event>> = Mutex::new(Vec::new());
    static ref PROCESSING: Mutex<Option<Event>> = Mutex::new(None);
    pub static ref CARDSTACK: Mutex<Vec<CardRequest>> = Mutex::new(Vec::new());
}

struct Event {
    event_type: EventType,
    attack_intent: Option<AttackIntent>,
    resolver: Box<dyn event_type::EventResolver + Send>,
    source: Option<Entity>,
    target_tiles: Arc<Vec<Point>>,
    invokes_reaction: bool,
}

pub fn add_event(
    event_type: &EventType,
    intent: Option<AttackIntent>,
    source: Option<Entity>,
    range: &RangeType,
    loc: Point,
    invokes_reaction: bool,
) {
    let mut stack = STACK.lock().expect("Failed to lock STACK");
    let event = Event {
        event_type: *event_type,
        attack_intent: intent,
        resolver: event_type::get_resolver(event_type),
        source,
        target_tiles: Arc::new(range_type::resolve_range_at(range, loc)),
        invokes_reaction,
    };

    stack.push(event);
}

pub fn add_damage_event(intent: &AttackIntent, source: Option<Entity>, invokes_reaction: bool) {
    let mut stack = STACK.lock().expect("Failed to lock STACK");

    let damage_event = EventType::Damage {
        amount: move_type::get_intent_power(intent),
    };
    let range = &move_type::get_attack_shape(&intent.main);
    let resolver = event_type::get_resolver(&damage_event);

    let event = Event {
        event_type: damage_event,
        attack_intent: Some(*intent),
        resolver,
        source,
        target_tiles: Arc::new(range_type::resolve_range_at(range, intent.loc)),
        invokes_reaction,
    };

    stack.push(event);
}

pub fn add_particle_event(position: Point, color: rltk::RGB, lifetime: f32) {
    add_event(
        &EventType::ParticleSpawn {
            request: crate::ParticleRequest {
                position,
                color,
                symbol: rltk::to_cp437('█'),
                lifetime,
            },
        },
        None,
        None,
        &crate::RangeType::Empty,
        Point::zero(),
        false,
    );
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

        return crate::RunState::HitPause {
            remaining_time: 600.0,
        };
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

// TODO: graphical effects need to be unentangled from the stack
pub fn process_stack_visual_only(ecs: &mut World) -> bool {
    loop {
        let event = {
            let mut stack = STACK.lock().expect("Failed to lock STACK");
            stack.pop()
        };

        match event {
            None => {
                return true;
            }
            Some(event) => match event.event_type {
                EventType::ParticleSpawn { .. } => {
                    process_event(ecs, event);
                }
                _ => {
                    STACK.lock().expect("Failed to lock STACK").push(event);
                    return false;
                }
            },
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
                None => {
                    {
                        // reset rolls, since there's no other attack
                        let mut intents = ecs.fetch_mut::<crate::IntentData>();
                        intents.hidden = false;
                        intents.rolls = (0, 0, 0, 0, false);
                    }

                    event
                        .resolver
                        .resolve(ecs, event.source, event.target_tiles.to_vec());
                }
                Some(stack_event) => {
                    if let Some(stack_source) = stack_event.source {
                        // TODO: this won't work if we want to allow enemies to counterattack
                        let stack_source_is_player = {
                            let player = ecs.fetch::<Entity>();
                            stack_source == *player
                        };

                        // If the stack event isn't from the player, the player didn't act, so there's no reaction to process
                        if !stack_source_is_player {
                            {
                                // replace the stack event if we're not using it
                                STACK
                                    .lock()
                                    .expect("Failed to lock STACK")
                                    .push(stack_event);
                            }

                            event
                                .resolver
                                .resolve(ecs, event.source, event.target_tiles.to_vec());

                            return;
                        }
                    }

                    match stack_event.attack_intent {
                        None => {
                            {
                                // replace the stack event if we're not using it
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
                            let speed_diff = compare_event_speed(ecs, &event, &stack_event);

                            let (first_event, second_event) = if speed_diff >= 0 {
                                println!("attacker wins the speed roll");
                                (event, stack_event)
                            } else {
                                println!("defender wins the speed roll");
                                (stack_event, event)
                            };

                            first_event.resolver.resolve(
                                ecs,
                                first_event.source,
                                first_event.target_tiles.to_vec(),
                            );

                            let can_interrupt = match first_event.event_type {
                                EventType::Damage { .. } => {
                                    let (def_guard_roll, atk_power_roll) = {
                                        let mut rng =
                                            ecs.fetch_mut::<rltk::RandomNumberGenerator>();
                                        let s3 = rng.range(0, GUARD_ROLL_RANGE);
                                        let s4 = rng.range(0, GUARD_ROLL_RANGE);

                                        let mut intents = ecs.fetch_mut::<crate::IntentData>();
                                        intents.rolls.2 = s3;
                                        intents.rolls.3 = s4;

                                        (s3, s4)
                                    };

                                    let (def, def_bonus_active) = if speed_diff >= 0 {
                                        (stack_intent, true)
                                    } else {
                                        (event_intent, false)
                                    };

                                    // compare power vs guard to determine if the defender can counter
                                    // only the defender can gain the guard bonus
                                    let mut def_guard =
                                        move_type::get_intent_guard(&def) + def_guard_roll;
                                    if def_bonus_active {
                                        def_guard += DEF_GUARD_BONUS;
                                    }
                                    let stun_power = speed_diff.abs() + atk_power_roll;

                                    stun_power > def_guard
                                }
                                _ => false,
                            };

                            if !can_interrupt {
                                second_event.resolver.resolve(
                                    ecs,
                                    second_event.source,
                                    second_event.target_tiles.to_vec(),
                                );
                            } else {
                                println!("defender is stunned!");
                            }
                        }
                    }
                }
            }
        }
    }
}

fn compare_event_speed(ecs: &mut World, attack_event: &Event, react_event: &Event) -> i32 {
    let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
    let atk_speed_roll = rng.range(0, SPEED_ROLL_RANGE);
    let def_speed_roll = rng.range(0, SPEED_ROLL_RANGE);

    // compare speed to determine which attack resolves first
    // also just unwrap the events here, we already pattern matched on them
    let atk_speed = move_type::get_intent_speed(&attack_event.attack_intent.unwrap())
        + atk_speed_roll
        + ATK_SPD_BONUS;
    let def_speed =
        move_type::get_intent_speed(&react_event.attack_intent.unwrap()) + def_speed_roll;

    let mut intents = ecs.fetch_mut::<crate::IntentData>();
    intents.hidden = false;
    intents.rolls.0 = atk_speed_roll;
    intents.rolls.1 = def_speed_roll;
    intents.rolls.4 = atk_speed >= def_speed;

    atk_speed - def_speed
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
