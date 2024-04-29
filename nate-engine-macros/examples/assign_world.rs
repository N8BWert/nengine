//!
//! Assignment World to Demonstrate Assigning Component values before systems
//! 

use nate_engine_macros::{system, world};

#[world(singular=[canvas])]
pub struct AssignWorld {
    canvas: [[bool; 10]; 10],
}

#[system(world=AssignWorld, _write=[canvas=[[true; 10]; 10]])]
fn print_true_canvas() {
    for y in 0..10 {
        for x in 0..10 {
            if canvas[y][x] {
                print!("X");
            } else {
                print!("_");
            }
        }
        print!("\n");
    }
}

#[system(world=AssignWorld, _write=[canvas=[[false; 10]; 10]])]
fn print_false_canvas() {
    for y in 0..10 {
        for x in 0..10 {
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
    let world = AssignWorld::new();

    {
        let mut world = world.write().unwrap();
        world.set_canvas([[false; 10]; 10]);
    }

    print_true_canvas(world.clone());
    print_false_canvas(world.clone());
}