//!
//! Big World to Demonstrate Having a Large number of components
//! that are used by one system
//! 

use nengine_macros::{system, world};

use rand::random;

#[world]
pub struct BigWorld {
    position: (f32, f32),
    velocity: (f32, f32),
    acceleration: (f32, f32),
    jerk: (f32, f32),
    health: u32,
    alive: bool,
}

#[system(world=BigWorld, read=[position, velocity, acceleration, jerk, health, alive])]
fn log_world() {
    println!(
        "Entity{{ position: {:?}, velocity: {:?}, acceleration: {:?}, jerk: {:?}, health: {:?}, alive: {:?} }}",
        position,
        velocity,
        acceleration,
        jerk,
        health,
        alive,
    )
}

#[system(world=BigWorld, read=[jerk], write=[acceleration, velocity, position])]
fn update_position() {
    *acceleration = (acceleration.0 + jerk.0, acceleration.1 + jerk.1);
    *velocity = (velocity.0 + acceleration.0, velocity.1 + acceleration.1);
    *position = (position.0 + velocity.0, position.1 + velocity.1);
}

#[system(world=BigWorld, write=[health, alive])]
pub fn update_health() {
    *health = health.saturating_sub(1);
    if *health == 0 {
        *alive = false;
    }
}

fn main() {
    let world = BigWorld::new();
    {
        let mut world = world.write().unwrap();
        let entity_ids = world.add_entities(100_000);

        let positions = entity_ids.iter().map(|_v| (0.0 ,0.0)).collect();
        world.set_positions(&entity_ids, positions);

        let velocities = entity_ids.iter().map(|_v| (0.0, 0.0)).collect();
        world.set_velocitys(&entity_ids, velocities);

        let accelerations = entity_ids.iter().map(|_v| (0.0, 0.0)).collect();
        world.set_accelerations(&entity_ids, accelerations);

        let jerks = entity_ids.iter().map(|_v| (random::<f32>(), random::<f32>())).collect();
        world.set_jerks(&entity_ids, jerks);

        let healths = entity_ids.iter().map(|_v| random::<u32>() % 20).collect();
        world.set_healths(&entity_ids, healths);

        let alives = entity_ids.iter().map(|_v| true).collect();
        world.set_alives(&entity_ids, alives);
    }

    log_world(world.clone());
    for _ in 0..10 {
        update_position(world.clone());
        update_health(world.clone());
    }
    log_world(world.clone());
}