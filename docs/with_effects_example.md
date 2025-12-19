```rust
# use bevy::prelude::*;
# use bevy_alchemy::*;
#
# #[derive(Component, Default)]
# struct MyEffect;
#
# fn main() {
#   let mut world = World::new();
#   let target = world.spawn_empty().id();
#   let mut commands = world.commands();
commands.entity(target).with_effects(|effects| {
    effects.spawn(EffectBundle {
        name: Name::new("EffectA"),
        bundle: MyEffect,
        ..default()
    });

    effects.spawn(EffectBundle {
        name: Name::new("EffectB"),
        bundle: MyEffect,
        ..default()
    });
});
# }
```