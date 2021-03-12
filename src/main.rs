#[macro_use]
extern crate lazy_static;

rltk::embedded_resource!(FONT, "../resources/Zilk-16x16.png");
rltk::embedded_resource!(ICONS, "../resources/custom_icons.png");

use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
mod deck;
mod events;
mod gamelog;
mod gui;
mod map;
mod move_type;
mod player;
mod spawner;
mod sys_ai;
mod sys_attack;
mod sys_death;
mod sys_mapindex;
mod sys_movement;
mod sys_particle;
mod sys_turn;
mod sys_visibility;

pub use components::*;
pub use events::*;
pub use map::{Map, TileType};
pub use move_type::*;
pub use sys_particle::{CardRequest, ParticleBuilder, ParticleRequest};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    Targetting { attack_type: AttackType },
    Running,
}

pub struct State {
    ecs: World,
    tick: i32,
    cursor: rltk::Point,
    tab_targets: Vec<rltk::Point>,
    tab_index: usize,
    attack_modifier: Option<AttackType>,
}

impl State {
    fn run_systems(&mut self) {
        self.tick += 1;

        sys_ai::AiSystem.run_now(&self.ecs);
        sys_turn::TurnSystem.run_now(&self.ecs);

        sys_movement::MovementSystem.run_now(&self.ecs);
        sys_attack::AttackSystem.run_now(&self.ecs);

        // events are processed after everything relevant is added (only attacks currently)
        events::process_stack(&mut self.ecs);

        // index needs to run after movement so blocked tiles are updated
        sys_mapindex::MapIndexSystem.run_now(&self.ecs);

        // death needs to run after attacks so bodies are cleaned up
        sys_death::DeathSystem.run_now(&self.ecs);

        sys_visibility::VisibilitySystem.run_now(&self.ecs);
        sys_particle::ParticleSpawnSystem.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // cleanup
        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        sys_particle::cleanup_particles(&mut self.ecs, ctx);

        // draw map + gui
        gui::draw_map(&self.ecs, ctx);
        gui::draw_renderables(&self.ecs, ctx);
        gui::draw_cards(&self.ecs, ctx);
        gui::draw_ui(&self.ecs, ctx);
        gui::draw_hand(&self.ecs, ctx);

        let mut next_status;

        // wrapping to limit borrowed lifetimes
        {
            // let player = self.ecs.fetch::<Entity>();
            // let can_act = self.ecs.read_storage::<CanActFlag>();
            // match can_act.get(*player) {
            //     None => ctx.print(30, 1, format!("OPPONENT TURN {}", self.tick)),
            //     Some(_) => ctx.print(30, 1, format!("YOUR TURN {}", self.tick)),
            // }

            // get the current RunState
            next_status = *self.ecs.fetch::<RunState>();
        }

        match next_status {
            RunState::AwaitingInput => {
                gui::update_controls_text(&self.ecs, ctx, &next_status);
                next_status = player::player_input(self, ctx);
            }
            RunState::Targetting { attack_type } => {
                gui::update_controls_text(&self.ecs, ctx, &next_status);
                let mut range = get_attack_range(&attack_type);
                if let Some(modifier) = self.attack_modifier {
                    range += crate::move_type::get_attack_range(&modifier);
                }

                let result = player::ranged_target(self, ctx, range);
                match result.0 {
                    player::SelectionResult::Canceled => {
                        let mut deck = self.ecs.fetch_mut::<deck::Deck>();
                        deck.selected = -1;
                        next_status = RunState::AwaitingInput;
                    }
                    player::SelectionResult::NoResponse => {}
                    player::SelectionResult::Selected => {
                        {
                            let mut deck = self.ecs.fetch_mut::<deck::Deck>();
                            deck.discard_selected();

                            let shape = crate::move_type::get_attack_shape(&attack_type);
                            if shape == crate::RangeType::Empty {
                                self.attack_modifier = Some(attack_type);
                            } else {
                                let target = result.1.unwrap();
                                let intent = crate::move_type::get_attack_intent(
                                    &attack_type,
                                    target,
                                    self.attack_modifier,
                                );
                                let player = self.ecs.fetch::<Entity>();
                                let mut attacks = self.ecs.write_storage::<AttackIntent>();

                                attacks
                                    .insert(*player, intent)
                                    .expect("Failed to insert attack from Player");

                                self.attack_modifier = None;
                            }
                        }

                        next_status = RunState::Running;
                        player::end_turn_cleanup(&mut self.ecs);
                    }
                }
            }
            RunState::Running => {
                // uncomment while loop to skip rendering intermediate states
                while next_status == RunState::Running {
                    self.run_systems();
                    // std::thread::sleep(std::time::Duration::from_millis(100));
                    next_status = *self.ecs.fetch::<RunState>();
                }
            }
        }

        let mut status_writer = self.ecs.write_resource::<RunState>();
        *status_writer = next_status;
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    rltk::link_resource!(FONT, "resources/Zilk-16x16.png");
    rltk::link_resource!(ICONS, "resources/custom_icons.png");

    let context = RltkBuilder::simple(gui::CONSOLE_WIDTH, gui::CONSOLE_HEIGHT)?
        .with_title("Roguelike Tutorial")
        .with_font("Zilk-16x16.png", 16, 16)
        .with_font("custom_icons.png", 16, 16)
        .with_simple_console_no_bg(gui::CONSOLE_WIDTH, gui::CONSOLE_HEIGHT, "Zilk-16x16.png")   // main layer
        .with_sparse_console_no_bg(gui::CONSOLE_WIDTH, gui::CONSOLE_HEIGHT, "custom_icons.png") // custom icons
        .with_sparse_console(gui::CONSOLE_WIDTH, gui::CONSOLE_HEIGHT, "Zilk-16x16.png")         // control line
        .build()
        .expect("Failed to build console");

    let mut gs = State {
        ecs: World::new(),
        tick: 0,
        cursor: rltk::Point::zero(),
        tab_targets: Vec::new(),
        tab_index: 0,
        attack_modifier: None,
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<CanActFlag>();
    gs.ecs.register::<CanReactFlag>();
    gs.ecs.register::<Schedulable>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<CardLifetime>();
    gs.ecs.register::<BlocksTile>();

    gs.ecs.register::<Health>();
    gs.ecs.register::<DeathTrigger>();
    gs.ecs.register::<AttackIntent>();
    gs.ecs.register::<MoveIntent>();
    gs.ecs.register::<Moveset>();

    gs.ecs.insert(RunState::Running);
    gs.ecs.insert(sys_particle::ParticleBuilder::new());

    let mut rng = rltk::RandomNumberGenerator::new();
    let mut map = map::build_rogue_map(gui::MAP_W, gui::MAP_H, &mut rng);
    let player_pos = map.rooms[0].center();

    let mut spawner =
        spawner::Spawner::new(&mut gs.ecs, &mut map.blocked_tiles, &mut rng, gui::MAP_W);

    for room in map.rooms.iter().skip(1) {
        spawner.build(&room, 0, 4, spawner::build_mook);
        spawner.build(&room, 0, 3, spawner::build_barrel);
    }

    gs.ecs.insert(map);

    let log = gamelog::GameLog {
        entries: vec!["Hello world!".to_string()],
    };
    gs.ecs.insert(log);

    let player = spawner::build_player(&mut gs.ecs, player_pos);
    gs.ecs.insert(player);

    let deck = deck::Deck::new(vec![
        AttackType::Super,
        AttackType::Super,
        AttackType::Punch,
        AttackType::Punch,
        AttackType::Punch,
        AttackType::Punch,
        AttackType::Sweep,
        AttackType::Sweep,
    ]);
    gs.ecs.insert(deck);

    gs.ecs.insert(rng);

    rltk::main_loop(context, gs)
}
