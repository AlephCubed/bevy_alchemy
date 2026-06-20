# Changelog

## 0.6.0 (2026-06-19)

- Updated to Bevy 0.19.
- Moved primary repository to Tangled.
- Moved changelog in-repo.

## 0.5.0 - Merge System Overhaul (2026-04-04)

The effect merging system has be overhauled,
making temporary entities fully isolated from the main world.

This change also makes merge functions themselves more ergonomic,
providing an `EntityRef` instead of an entity ID.
Note that **this swaps the direction of the merge**!
The results of merge logic should now be applied to the ***existing** component*.

### Migration Guide

Before:

```rust
fn merge_my_effect(mut new: EntityWorldMut, outgoing: Entity) {
    let outgoing = new.world().get::<MyEffect>(outgoing).unwrap().clone();
    let mut new = new.get_mut::<MyEffect>().unwrap();
    new.0 += outgoing.0;
}
```

After:

```rust
fn merge_my_effect(mut existing: EntityWorldMut, incoming: EntityRef) {
    let mut existing = existing.get_mut::<MyEffect>().unwrap();
    let incoming = incoming.get::<MyEffect>().unwrap();
    existing.0 += incoming.0;
}
```

## 0.4.0 - Fully Generic Bundles (2026-04-01)

- Removed `EffectBundle`.
 Instead, effects can be applied using any bundle that implements `Clone`.
- Added benchmarks for stack and insert modes.

### Migration Guide

#### `with_effect`

Before:

```rust
commands.entity(target).with_effect(EffectBundle {
    name: Name::new("Effect"),
    bundle: MyEffect,
    ..default()
});
```

After:

```rust
commands.entity(target).with_effect((
    Name::new("Effect"), 
    MyEffect,
));
```

#### `EffectedBy::spawn`

Instead of using an `EffectBundle`, an `Effect` wrapper has been added
(equivalent to Bevy's [`Spawn`](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Spawn.html)).

Before:

```rust
commands.spawn((
    Name::new("Target"),
    EffectedBy::spawn(EffectBundle {
        name: Name::new("Effect"),
        bundle: MyEffect,
        ..default()
    }),
));
```

After:

```rust
commands.spawn((
    Name::new("Target"),
    EffectedBy::spawn(Effect((
        Name::new("Effect"), 
        MyEffect,
    ))),
));
```

## 0.3.0 (2026-01-28)

- Added `get_timer`, `get_mode`, and mutable equivelents to `EffectTimer` trait.
- Added default implementation for `EffectTimer::merge`.
- Fixed a warning message referencing old `StatusEffectPlugin` instead of `AlchemyPlugin`.
- Added `Delay::trigger_immediately` builder method.
  - This is now used in both poison examples.
- Removed `register_timer_merge_functions` in favor of `merge_effect_timer`:

 ```rust
 // Before:
 register_timer_merge_functions(&mut registry);
 // After:
 registry
     .register::<Lifetime>(merge_effect_timer::<Lifetime>)
     .register::<Delay>(merge_effect_timer::<Delay>);
 ```

- Made `merge_effect_stacks` public.
- Implemented `Add<Self>`, `AddAssign<Self>`, `From<u8>`, and `Into<u8>` for `EffectStacks`.
- Various documentation and examples cleanup.

## 0.2.1 (2026-01-25)

- Added `EffectStacks` component.
- Added `poison_falloff` example.

## 0.2.0 (2026-01-14)

- Update to Bevy 0.18.

## 0.1.0 (2025-12-26)

- Initial release.
