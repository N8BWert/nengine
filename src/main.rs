use nengine_macros::{world, system};

pub struct Sprite {
    _test: u32,
}

#[world]
pub struct CreatedWorld {
    position: (f32, f32),
    player_velocity: (f32, f32),
    sprite: Sprite,
}

// Above Should Look Like
pub struct ExpectedWorld {
    entities: std::sync::Arc<std::sync::RwLock<std::vec::Vec<u32>>>,

    pub position: std::sync::Arc<std::sync::RwLock<std::vec::Vec<std::option::Option<(f32, f32)>>>>,
    pub player_velocity: std::sync::Arc<std::sync::RwLock<std::vec::Vec<std::option::Option<(f32, f32)>>>>,
    pub sprite: std::sync::Arc<std::sync::RwLock<std::vec::Vec<std::option::Option<Sprite>>>>,
}

impl ExpectedWorld {
    pub fn new() -> Self {
        Self {
            entities: std::sync::Arc::new(std::sync::RwLock::new(std::vec::Vec::new())),
            position: std::sync::Arc::new(std::sync::RwLock::new(std::vec::Vec::new())),
            player_velocity: std::sync::Arc::new(std::sync::RwLock::new(std::vec::Vec::new())),
            sprite: std::sync::Arc::new(std::sync::RwLock::new(std::vec::Vec::new())),
        }
    }

    pub fn add_entity(&mut self) -> u32 {
        let entity_id = self.entities.read().unwrap().len() as u32;
        self.entities.write().unwrap().push(entity_id);
        self.position.write().unwrap().push(None);
        self.player_velocity.write().unwrap().push(None);
        self.sprite.write().unwrap().push(None);
        entity_id
    }

    pub fn set_position(&mut self, entity_id: u32, position: (f32, f32)) {
        self.position.write().unwrap()[entity_id as usize] = Some(position);
    }

    pub fn clear_position(&mut self, entity_id: u32) {
        self.position.write().unwrap()[entity_id as usize] = None;
    }
}

#[system(world=CreatedWorld, read=[player_velocity], write=[position])]
fn position_update_system() {
    *position = Some((
        position.unwrap().0 + player_velocity.unwrap().0,
        position.unwrap().1 + player_velocity.unwrap().1,
    ))
}

fn expected_position_update_system(world: std::sync::Arc<CreatedWorld>) {
    let player_velocity = world.player_velocity.read().unwrap();
    let mut position = world.position.write().unwrap();
    for (player_velocity, position) in player_velocity.iter().zip(position.iter_mut()).filter(|v| v.0.is_some() && v.1.is_some()) {
        *position = Some((
            position.unwrap().0 + player_velocity.unwrap().0,
            position.unwrap().1 + player_velocity.unwrap().1,
        ))
    }
}

fn main() {
    let _world = CreatedWorld::new();
}
