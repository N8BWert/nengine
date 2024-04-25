# NEngine

## Description

NEnging is a proof of concept project I'm working on when I'm bored.  The main concept is that NEngine is a game engine backed by the entity component system (ECS) model.  I'm probably going to use it to make a few tui games or something but I'd highly recommend against anyone else using as it is purely a test and most real projects should probably use something like bevy.

## Goal

The main goal of the project is to make the ECS system be created incredibly easily via a few macros as follows:

### Components

The components macro should make it easy to define the list of components to be used in a project.  Probably something like as follows:

```rust
#[world]
pub struct World {
    position: (f32, f32),
    player_velocity: (f32, f32),
    object_velocity: (f32, f32),
    sprite: Sprite,
}
```

Which likely translates to something similar to:

```rust
pub struct World {
    entities: Arc<RwLock<Vec<u32>>>,

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
fn position_update_system(ctx: position_update_system::Context) {
    *ctx.position += ctx.object_velocity;
}
```

Which would likely expand to:

```rust
fn position_update_system(world: Arc<DinosaurWorld>) {
    let mut positions = world.positions.read().unwrap();
    let object_velocities = world.object_velocities.write().unwrap();
    for (position, velocity) in positions.iter_mut().zip(object_velocities.iter()).filter(|v| v.0.is_some() && v.1.is_some()) {
        *position += velocity;
    }
}
```

It would also likely be useful to add some sort of filter to allow users to filter not-only that some component exists, but also that the component has some value.  For example take the following situation:

```rust
#[world(
    components=[
        (position, (u32, u32), dense),
        (damage_zone, (u32, u32), dense),
        (health, u32, health)
    ]
)]
pub struct World {}

#[system(
    write=[health],
    read=[position, damage_zone],
    filter="position == damage_zone"
)]
fn position_damage_system(ctx: position_damage_system::Context) {
    ctx.health -= 1;
}
```

Which would expand to:

```rust
pub struct World {
    entities: Arc<Vec<u32>>,

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

I'm not currently 100% certain I can do this because I think I'll need to write some runtime or something to actually insert the world, but this is kind of my goal.
