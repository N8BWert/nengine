//!
//! World that has some system that uses the filter part of the
//! system macro
//! 

use nate_engine_macros::{system, world};

use rand::random;

#[world]
pub struct FilterWorld {
    health: u32,
}

#[system(world=FilterWorld, read=[health], filter=[*health < 2])]
fn log_dying_entities() {
    println!("An Entity Is Dying");
}

#[system(world=FilterWorld, write=[health])]
fn decrease_health() {
    *health = health.saturating_sub(1);
}

fn main() {
    let world = FilterWorld::new();

    {
        let mut world = world.write().unwrap();

        let entity_ids = world.add_entities(20);

        let healths = entity_ids.iter().map(|_v| random::<u32>() % 10).collect();
        world.set_healths(&entity_ids, healths);
    }

    log_dying_entities(world.clone());
    println!("Doing Damage");

    for _ in 0..10 {
        decrease_health(world.clone());
    }
    
    log_dying_entities(world);
}