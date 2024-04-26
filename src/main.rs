//!
//! Example Program completing the same ecs-toy example I did earlier on my GitHub, but now with my engine
//! 

use std::{io::{stdout, Result, Stdout}, time::{SystemTime, UNIX_EPOCH, Duration}};

use clap::Parser;

use nengine::{Engine, Renderer};
use nengine_macros::{system, world};

use rand::random;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::Canvas, Block, Borders},
};

const WIDTH: isize = 212;
const MIN_X: isize = -106;
const MAX_X: isize = 105;

const HEIGHT: isize = 50;
const MIN_Y: isize = -25;
const MAX_Y: isize = 24;

const MAX_HEALTH: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Dead,
    Low,
    Medium,
    High,
}

#[world(singular=[living_entities, canvas])]
pub struct ToyWorld {
    position: (isize, isize),
    velocity: (isize, isize),
    acceleration: (isize, isize),
    health: usize,
    health_changes: isize,

    living_entities: usize,
    canvas: [[Status; WIDTH as usize]; HEIGHT as usize],
}

#[system(world=ToyWorld, read=[velocity], write=[position])]
fn position_update_system() {
    *position = (
        (position.0 + velocity.0).clamp(MIN_X, MAX_X),
        (position.1 + velocity.1).clamp(MIN_Y, MAX_Y)
    );
}

#[system(world=ToyWorld, read=[acceleration], write=[velocity])]
fn velocity_update_system() {
    *velocity = (velocity.0 + acceleration.0, velocity.1 + acceleration.1);
}

#[system(world=ToyWorld, write=[acceleration])]
fn acceleration_update_system() {
    let left_right = match random::<usize>() % 4 {
        0 => 1,
        1 => -1,
        _ => 0,
    };

    let up_down = match random::<usize>() % 4 {
        0 => 1,
        1 => -1,
        _ => 0,
    };

    *acceleration = (left_right, up_down);
}

#[system(world=ToyWorld, _write=[canvas])]
fn clear_canvas_system() {
    *canvas = [[Status::Dead; WIDTH as usize]; HEIGHT as usize];
}

#[system(world=ToyWorld, read=[position, health], _write=[canvas])]
fn update_canvas_system() {
    let x = (position.0.clamp(MIN_X, MAX_X) + WIDTH / 2) as usize;
    let y = (position.1.clamp(MIN_Y, MAX_Y) + HEIGHT / 2) as usize;

    match health {
        7.. => canvas[y][x] = Status::High,
        4..=6 => canvas[y][x] = Status::Medium,
        1..=3 => canvas[y][x] = Status::Low,
        _ => (),
    }
}

#[system(world=ToyWorld, read=[health_changes], write=[health])]
fn health_update_system() {
    *health = health.saturating_add_signed(*health_changes).clamp(0, MAX_HEALTH);
}

#[system(world=ToyWorld, write=[health_changes])]
fn health_changes_system() {
    if random() {
        *health_changes = random::<isize>() % 2;
    } else {
        *health_changes = -random::<isize>() % 2;
    }
}

#[system(world=ToyWorld, _write=[living_entities])]
fn alive_entities_display_system() {
    *living_entities = world.health.read().unwrap().iter().map(|v| v.is_some() && v.unwrap() > 0).count();
}

pub struct ToyTerminalRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    last_render: u128,
}

impl ToyTerminalRenderer {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        Self {
            terminal,
            last_render: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        }
    }
}

impl Renderer<ToyWorld> for ToyTerminalRenderer {
    type Error = String;

    fn render(&mut self, world: std::sync::Arc<std::sync::RwLock<ToyWorld>>) -> std::prelude::v1::Result<(), Self::Error> {
        let world = world.read().unwrap();

        let then = self.last_render;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        self.last_render = now;
        let frame_rate = if now - then == 0 {
            60
        } else {
            1000 / (now - then)
        };

        let _err = self.terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                Canvas::default()
                    .block(
                        Block::default()
                        .borders(Borders::ALL)
                        .title(
                            format!(
                                "Living Entities: {} ------ ({} fps)",
                                (*world.living_entities.read().unwrap()).unwrap(),
                                frame_rate,
                            )
                        )
                    )
                    .background_color(Color::Black)
                    .x_bounds([0.0, WIDTH as f64])
                    .y_bounds([0.0, HEIGHT as f64])
                    .paint(|ctx| {
                        let canvas = (*world.canvas.read().unwrap()).unwrap();
                        for (y, row) in canvas.iter().enumerate() {
                            for (x, item) in row.iter().enumerate() {
                                match *item {
                                    Status::Dead => ctx.print(x as f64, y as f64, "_".gray()),
                                    Status::Low => ctx.print(x as f64, y as f64, "X".red()),
                                    Status::Medium => ctx.print(x as f64, y as f64, "X".yellow()),
                                    Status::High => ctx.print(x as f64, y as f64, "X".green()),
                                }
                            }
                        }
                    }),
                area
            )
        });

        if event::poll(Duration::from_millis(5)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press &&
                    key.code == KeyCode::Char('q') {
                    return Err("Interrupt Pressed".into());
                }
            }
        }

        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Number of Entities to Spawn
    #[arg(short, long, default_value_t = 100_000)]
    entities: usize,

    // Workers in the threadpool
    #[arg(short, long, default_value_t = 3)]
    workers: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let world = ToyWorld::new();
    {
        let mut world = world.write().unwrap();
        let entity_ids = world.add_entities(args.entities as u32);

        let positions = entity_ids.iter().map(|_v| (random::<isize>().clamp(MIN_X, MAX_X), random::<isize>().clamp(MIN_Y, MAX_Y))).collect();
        world.set_positions(&entity_ids, positions);

        let velocities = entity_ids.iter().map(|_v| (0, 0)).collect();
        world.set_velocitys(&entity_ids, velocities);

        let accelerations = entity_ids.iter().map(|_v| (0, 0)).collect();
        world.set_accelerations(&entity_ids, accelerations);

        let healths = entity_ids.iter().map(|_v| random::<usize>() % 100).collect();
        world.set_healths(&entity_ids, healths);

        let health_changes = entity_ids.iter().map(|_v| 0).collect();
        world.set_health_changess(&entity_ids, health_changes);

        world.set_living_entities(args.entities);
        world.set_canvas([[Status::Dead; WIDTH as usize]; HEIGHT as usize]);
    }

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut engine = Engine::new(
        1,
        args.workers,
        world,
        vec![
            (position_update_system, 100),
            (velocity_update_system, 100),
            (acceleration_update_system, 100),
            (update_canvas_system, 100),
            (health_update_system, 100),
            (alive_entities_display_system, 100),
        ],
        Box::new(ToyTerminalRenderer::new(terminal))
    );

    engine.run();

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
