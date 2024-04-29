//!
//! Test Program using a small world to test the functionality of the macros
//! 

use nate_engine_macros::{system, world};

#[world]
pub struct SmallWorld {
    position: (f32, f32),
    player_velocity: (f32, f32),
}

#[system(world=SmallWorld, read=[position])]
pub fn read_positions() {
    println!("Position: {:?}", position);
}

#[system(world=SmallWorld, write=[player_velocity])]
pub fn randomize_player_velocities() {
    if player_velocity.0 == 0.0 {
        player_velocity.0 = 1.0;
    } else {
        player_velocity.0 = 0.0;
    }

    if player_velocity.1 == 0.0 {
        player_velocity.1 = 1.0;
    } else {
        player_velocity.1 = 0.0;
    }

    println!("Player Velocity: {:?}", player_velocity);
}

#[system(world=SmallWorld, read=[player_velocity])]
fn read_velocities() {
    println!("Velocity: {:?}", player_velocity);
}

fn main() {
    let world = SmallWorld::new();
    {
        let mut world = world.write().unwrap();
        world.add_entities(4);
        world.set_positions(&vec![0, 1, 2, 3], vec![(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)]);
        world.set_player_velocitys(&vec![0, 1, 2, 3], vec![(3.0, 3.0), (2.0, 2.0), (1.0, 1.0), (0.0, 0.0)]);
    }

    read_positions(world.clone());
    read_velocities(world.clone());
    randomize_player_velocities(world.clone());
    read_velocities(world.clone());
}