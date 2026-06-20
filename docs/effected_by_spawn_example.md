```rust
# use bevy::prelude::*;
# use bevy_alchemy::*;
#
# #[derive(Component, Default, Clone)]
# struct MyEffect;
#
# fn main() {
#   let mut world = World::new();
#   let target = world.spawn_empty().id();
#   let mut commands = world.commands();
commands.spawn((
    Name::new("Target"),
    EffectedBy::spawn(
        Effect((Name::new("Effect"), MyEffect))
    ),
));
# }
```
