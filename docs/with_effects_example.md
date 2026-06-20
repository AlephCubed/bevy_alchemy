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
commands.entity(target).with_effects(|effects| {
    effects.spawn((Name::new("EffectA"), MyEffect));
    effects.spawn((Name::new("EffectB"), MyEffect));
});
# }
```
