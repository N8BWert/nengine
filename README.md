# Nate-Engine

## Description

Nate-Engine is a proof of concept project I'm working on when I'm bored.  The main concept is that Nate-Engine is a game engine backed by the entity component system (ECS) model.  I'm probably going to use it to make a few tui games or something but I'd highly recommend against anyone else using as it is purely a test and most real projects should probably use something like bevy.

## Goal

The main goal of the project is to make the ECS system be created incredibly easily via a few macros as follows:

### Components

The components macro should make it easy to define the list of components to be used in a project.  It might also be useful to have items that are singular and not duplicated for every entity. Probably something like as follows:

```rust
#[world(singular=[canvas])]
pub struct World {
    position: (f32, f32),
    player_velocity: (f32, f32),
    object_velocity: (f32, f32),
    sprite: Sprite,
    canvas: [[bool; 10]; 10],
}
```

Which translates into:

```rust
pub struct World {
    entities: Arc<RwLock<Vec<u32>>>,

    pub canvas: Arc<RwLock<Option<[[bool; 10]; 10]>>>,

    pub position: Arc<RwLock<Vec<Option<(f32, f32)>>>>,
    pub player_velocity: Arc<RwLock<Vec<Option<(f32, f32)>>>>,
    pub object_velocity: Arc<RwLock<Vec<Option<(f32, f32)>>>>,
    pub sprite: Arc<RwLock<Vec<Option<Sprite>>>>,
}
```

### Systems

For systems, I'd like to make it such that the iterator + filter is auto generated so that in the proc-macro all the user has to specify is what fields should be present, plus an optional filter parameter.  For example:

```rust
#[system(world=DinosaurWorld, read=[object_velocity], write=position)]
fn position_update_system() {
    *position = (position.0 + object_velocity.0, position.1 + object_velocity.1);
}
```

Which would expand to:

```rust
fn position_update_system(world: Arc<DinosaurWorld>) {
    let object_velocity = world.object_velocity.read().unwrap();
    let mut position = world.positions.write().unwrap();
    for (object_velocity, position) in object_velocity.iter().zip(position.iter_mut()).filter(|v| v.0.is_some() && v.1.is_some()) {
        let object_velocity = object_velocity.as_ref().unwrap();
        let mut position = position.as_mut().unwrap();

        position = (position.0 + object_velocity.0, position.1 + object_velocity.1);
    }
}
```

It would also likely be useful to add some sort of filter to allow users to filter not-only that some component exists, but also that the component has some value.  For example take the following situation:

```rust
#[world]
pub struct World {
    position: (u32, u32),
    damage_zone: (u32, u32),
    health: u32,
}

#[system(
    world=World,
    write=[health],
    read=[position, damage_zone],
    filter="position == damage_zone"
)]
fn position_damage_system() {
    health -= 1;
}
```

Which expands to:

```rust
pub struct World {
    entities: Arc<RwLock<Vec<u32>>>,

    positions: Arc<RwLock<Vec<Option<(u32, u32)>>>>,
    damage_zones: Arc<RwLock<Vec<Option<(u32, u32)>>>>,
    healths: Arc<RwLock<Vec<Option<u32>>>>,
}

fn position_damage_system(world: Arc<World>) {
    let positions = world.positions.read().unwrap();
    let damage_zones = world.damage_zones.read().unwrap();
    let mut healths = world.healths.write().unwrap();
    for ((_position, _damage_zone), health) in positions.iter().zip(damage_zones.iter()).zip(healths.iter_mut()).filter(|v| v.0.0.is_some() && v.0.1.is_some() && v.1.is_some() && v.0.0 == v.0.1) {
        *health -= 1
    }
}
```

It is also possible to refer to singular components in systems by using _read and _write as their identifiers.  For example:

```rust
#[system(world=World, _read=[canvas], _write=[exit])]
fn read_canvas_and_exit() {
    ...
}
```

Would allows the user to access canvas (through reading) and write to exit
