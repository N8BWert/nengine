//!
//! Nate's Game Engine
//! 
//! # Creating Worlds
//! ```
//! #[world(singular=[canvas])]
//! pub struct World {
//!     position: (isize, isize),
//!     velocity: (isize, isize),
//!     health: usize,
//!     health_changes: isize,
//!     canvas: [[bool; 10]; 10],
//! }
//! ```
//! 
//! Creates a world named World, where every entity has the possibility of having a
//! position, velocity, health, and health_change, but the world has a singular canvas.
//! This will also generate getters and setters for each component as well as a default
//! initializer for the world.
//! 
//! # Declaring Systems
//! ```
//! #[system(world=World, read=[velocity], write=[position], _read=[game_state], _write=[canvas])]
//! pub fn some_system() {
//!     ...
//! }
//! ```
//! 
//! The above example creates a system named some_system that operates on a world of type World.
//! This system will operate on each entity that has a velocity and position component, such that
//! the entity's positions are mutable and their velocities are immutable.  The world game_state is
//! also readable and the world canvas is writable.
//! 
//! # Examples
//! 
//! One example of using the engine is accessible [here](examples/toy_example.rs).
//! An even simpler example is as follows:
//! ```
//! use nate_engine::{Engine, Renderer, system, world};
//! 
//! // Game State Enum
//! #[derive(Clone, Copy, Debug, PartialEq, Eq)]
//! pub enum GameState {
//!     Paused,
//!     Playing,
//!     Stopped,
//! }
//! 
//! // World where every entity can have a position, velocity, health, and health_change and
//! // canvas and game_state are singular components (i.e. components on the world)
//! #[world(singular=[canvas, game_state])]
//! pub struct ExampleWorld {
//!     position: (isize, isize),
//!     velocity: (isize, isize),
//!     health: usize,
//!     health_changes: isize,
//! 
//!     canvas: [[bool; 10]; 10],
//!     game_state: GameState,
//! }
//! 
//! // If the game is playing, update the position of every entity with both and position and
//! // velocity based on the velocity.
//! #[system(world=ExampleWorld, read=[velocity], write=[position], _read=[game_state])]
//! pub fn position_update_system() {
//!     if game_state == GameState::Playing {
//!         *position = (position.0 + velocity.0, position.1 + velocity.1);
//!     }
//! }
//! 
//! // If the game is playing, update the health of every entity with a health change
//! #[system(world=ExampleWorld, read=[health_changes], write=[health], _read=[game_state])]
//! pub fn health_update_system() {
//!     if game_state == GameState::Playing {
//!         *health = health.saturating_add_signed(*health_changes);
//!     }
//! }
//! 
//! // Write the positions of entities to the canvas
//! #[system(world=ExampleWorld, read=[position], _write=[canvas = [[false; 10]; 10])]
//! pub fn canvas_update_system() {
//!     let x = position.0.clamp(0.0, 9.0) as usize;
//!     let y = position.1.clamp(0.0, 9.0) as usize;
//!     canvas[y][x] = true;
//! }
//! 
//! // Renderer that will print to the console
//! pub struct ExampleRenderer {}
//! 
//! // Make sure the renderer is compatible with the game engine
//! impl Renderer<ExampleWorld> for ExampleRenderer {
//!     type Error = String;
//! 
//!     fn render(&mut self, world: Arc<RwLock<ExampleWorld>>) -> Result<(), Self::Error> {
//!         let world = world.read.unwrap();
//!         let canvas = (*world.canvas.read().unwrap()).unwrap();
//!         
//!         
//!         for y in 0..10 {
//!             for x in 0..10 {
//!                 if canvas[y][x] {
//!                     print!("X");
//!                 } else {
//!                     print!("_");
//!                 }
//!             }
//!             print!("\n");
//!         }
//! 
//!         // Handle Ctrl-c or Events to exit the loop
//!         Ok(())
//!     }
//! }
//! 
//! // The Frame Rate for the Rendering Engine
//! const FRAME_RATE: u32 = 30;
//! // The Number of CPU workers to Compute Systems (1 will be reserved for the rendering engine)
//! const WORKERS: usize = 2;
//! 
//! fn main() {
//!     let world = ExampleWorld::new();
//! 
//!     {
//!         let mut world = world.write().unwrap();
//!         // Create Entities in the World
//!         let entity_ids = world.add_entities(100);
//! 
//!         // Add Components to Entities
//!         world.set_positions(&entity_ids, ...);
//!         ...
//! 
//!         // Initialize Singular Components
//!         world.set_game_state(GameState::Playing);
//!         world.set_canvas([[false; 10]; 10]);
//!     }
//! 
//!     let mut engine = Engine::new(
//!         FRAME_RATE,
//!         WORKERS,
//!         world,
//!         vec![
//!             // position update system will run every 100ms
//!             (position_update_system, 100_000),
//!             // health update system will run every 1s
//!             (health_update_system, 1_000_000),
//!             // canvas update system will run every 100ms
//!             (canvas_update_system, 100_000),
//!         ],
//!         Box::new(ExampleRenderer{}),
//!     );
//! 
//!     engine.run();
//! }
//! ```
//! 

#[allow(rustdoc::invalid_rust_codeblocks)]

/// Re-export of Nate's Engine Core
pub use nate_engine_core::{Engine, Renderer};
/// Re-export of Nate's Engine Macros
pub use nate_engine_macros::{world, system};
