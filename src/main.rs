#[macro_use]
extern crate lazy_static;

use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
mod events;
mod gamelog;
mod gui;
mod map;
mod move_type;
mod player;
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

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    Running,
}

pub struct State {
    ecs: World,
    tick: i32,
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
        ctx.set_active_console(1);
        ctx.cls();
        sys_particle::cleanup_particles(&mut self.ecs, ctx);

        // draw map + gui
        gui::draw_map(&self.ecs, ctx);
        gui::draw_renderables(&self.ecs, ctx);
        gui::draw_cards(&self.ecs, ctx);
        gui::draw_ui(&self.ecs, ctx);

        let mut next_status;
        // wrapping to limit borrowed lifetimes
        {
            let player = self.ecs.fetch::<Entity>();
            let can_act = self.ecs.read_storage::<CanActFlag>();
            match can_act.get(*player) {
                None => ctx.print(30, 1, format!("OPPONENT TURN {}", self.tick)),
                Some(_) => ctx.print(30, 1, format!("YOUR TURN {}", self.tick)),
            }

            // get the current RunState
            next_status = *self.ecs.fetch::<RunState>();
        }

        match next_status {
            RunState::AwaitingInput => {
                next_status = player::player_input(self, ctx);
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

    const WIDTH: i32 = 80;
    const HEIGHT: i32 = 50;
    const CONSOLE_HEIGHT: i32 = HEIGHT + 7;

    let context = RltkBuilder::simple(WIDTH, CONSOLE_HEIGHT)?
        .with_title("Roguelike Tutorial")
        .with_font("terminal8x8.png", 8, 8)
        .with_simple_console_no_bg(WIDTH, CONSOLE_HEIGHT, "terminal8x8.png")
        .build()
        .expect("Failed to build console");

    let mut gs = State {
        ecs: World::new(),
        tick: 0,
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

    let map = map::build_rogue_map(WIDTH, HEIGHT);
    let player_pos = map.rooms[0].center();

    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center().to_tuple();
        let _enemy = gs
            .ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                symbol: rltk::to_cp437('x'),
                fg: RGB::named(rltk::LIGHT_BLUE),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Schedulable {
                current: 0,
                base: 24,
                delta: 4,
            })
            .with(Viewshed {
                visible: Vec::new(),
                dirty: true,
                range: 6,
            })
            .with(BlocksTile)
            .with(Health { current: 5, max: 5 })
            .with(Moveset { moves: vec![] })
            .build();
    }

    gs.ecs.insert(map);

    let log = gamelog::GameLog {
        entries: vec!["Hello world!".to_string()],
    };
    gs.ecs.insert(log);

    let player = gs
        .ecs
        .create_entity()
        .with(Position {
            x: player_pos.x,
            y: player_pos.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
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
        .build();
    gs.ecs.insert(player);

    let _explosive_barrel = gs
        .ecs
        .create_entity()
        .with(Position {
            x: player_pos.x - 1,
            y: player_pos.y - 1,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('#'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(BlocksTile)
        .with(Health { current: 2, max: 2 })
        .with(DeathTrigger {
            event: EventType::Damage { amount: 1 },
            range: RangeType::Square { size: 1 },
        })
        .build();

    rltk::main_loop(context, gs)
}
