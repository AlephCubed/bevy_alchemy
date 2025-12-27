# Bevy Alchemy

[![Version](https://img.shields.io/crates/v/bevy_alchemy)](https://crates.io/crates/bevy_alchemy)
[![Docs](https://img.shields.io/docsrs/bevy_alchemy)](https://docs.rs/bevy_alchemy)
![License](https://img.shields.io/crates/l/bevy_alchemy)

An experimental, status effects-as-entities system for Bevy.

### Applying Effects
Effects can be applied using `with_effect` or `with_effects` (similar to `with_child` and `with_children` respectively).
```rust ignore
commands.entity(target).with_effect(EffectBundle {
    name: Name::new("Effect"),
    bundle: MyEffect,
    ..default()
});
```
They can also be added using spawn-style syntax.
```rust ignore
commands.spawn((
    Name::new("Target"),
    EffectedBy::spawn(EffectBundle {
        name: Name::new("Effect"),
        bundle: MyEffect,
        ..default()
    }),
));
```

### Effect Modes
For some effects it makes sense to allow stacking, so a single entity could be effected by an effect multiple times.
Other effects should only be applied once, either replacing or merging with the previous one. 
This behaviour can be selected using an effect's `MergeMode`, which has the following cases:

| Mode   | Behaviour                                                                               |
|--------|-----------------------------------------------------------------------------------------|
| Stack  | Multiple of the same effect can exist at once.                                          |
| Insert | New applications will overwrite the existing one.                                       |
| Merge  | New applications are merged with the existing one, using a configurable merge function. |

Effects are considered the same if they have the same name.

### Implementing Effects
Effects can be implemented using simple systems. Below is an excerpt from the poison example.
```rust ignore
/// Runs every frame and deals the poison damage.
fn deal_poison_damage(
    effects: Query<(&Effecting, &Delay, &Poison)>,
    mut targets: Query<&mut Health>,
) {
    for (target, delay, poison) in effects {
        // We wait until the delay finishes to apply the damage.
        if !delay.timer.is_finished() {
            continue;
        }

        // Skip if the target doesn't have health.
        let Ok(mut health) = targets.get_mut(target.0) else {
            continue;
        };

        // Otherwise, deal the damage.
        health.0 -= poison.damage;
    }
}
```

### Timers
Two timers are added by the crate: 
1. `Lifetime` - Despawns the effect when the timer ends.
2. `Delay` - A repeating timer used for the delay between effect applications.

### Bevy Version Compatibility

| Bevy   | Bevy Alchemy |
|--------|--------------|
| `0.17` | `0.1`        |