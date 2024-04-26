//!
//! World that uses the ignore parameter of the macro
//! 

use nengine_macros::{system, world};

use rand::random;

#[world(singular=[canvas])]
pub struct SingularWorld {
    canvas: [[bool; 10]; 10],
    position: (f32, f32),
    velocity: (f32, f32),
}

#[system(world=SingularWorld, read=[position], _write=[canvas])]
fn write_position_to_canvas() {
    let position = (position.0.clamp(0.0, 9.0) as usize, position.1.clamp(0.0, 9.0) as usize);
    canvas[position.1][position.0] = true;
}

#[system(world=SingularWorld, _read=[canvas])]
fn print_canvas() {
    for y in 0..canvas.len() {
        for x in 0..canvas[0].len() {
            if canvas[y][x] {
                print!("X");
            } else {
                print!("_");
            }
        }
        print!("\n");
    }
}

fn main() {
    let world = SingularWorld::new();
    {
        let mut world = world.write().unwrap();

        let entity_ids = world.add_entities(10);

        let positions = entity_ids.iter().map(|_v| (random::<f32>() * 10.0, random::<f32>() * 10.0)).collect();
        world.set_positions(&entity_ids, positions);

        world.set_canvas([[false; 10]; 10]);
    }

    println!("Before");
    print_canvas(world.clone());
    write_position_to_canvas(world.clone());
    println!("After");
    print_canvas(world.clone());
}