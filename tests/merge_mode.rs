//! Tests the behaviour of adding effects with each [`EffectMode`].

use bevy_alchemy::*;
use bevy_ecs::prelude::*;
use bevy_time::*;
use std::time::Duration;

#[derive(Component, Debug, Eq, PartialEq, Default, Clone)]
struct MyEffect(u8);

fn init_world() -> World {
    let mut world = World::new();

    let mut registry = EffectMergeRegistry::default();
    registry
        .register::<Lifetime>(merge_effect_timer::<Lifetime>)
        .register::<Delay>(merge_effect_timer::<Delay>);

    world.insert_resource(registry);

    world
}

#[test]
fn stack() {
    let mut world = init_world();

    let target = world.spawn_empty().id();

    world.commands().entity(target).with_effects(|effects| {
        effects.spawn(MyEffect(0));
        effects.spawn(MyEffect(1));
    });

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&world)
        .map(|c| c.0)
        .collect();

    assert!(effects.contains(&0));
    assert!(effects.contains(&1));
}

#[test]
fn insert() {
    let mut world = init_world();

    let target = world.spawn_empty().id();

    world
        .commands()
        .entity(target)
        .with_effect((EffectMode::Insert, MyEffect(0)));
    world
        .commands()
        .entity(target)
        .with_effect((EffectMode::Insert, MyEffect(1)));

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&world)
        .map(|c| c.0)
        .collect();

    assert!(!effects.contains(&0));
    assert!(effects.contains(&1));
}

#[test]
fn mixed() {
    let mut world = init_world();

    let target = world.spawn_empty().id();

    world.commands().entity(target).with_effect(MyEffect(0));
    world.commands().entity(target).with_effect(MyEffect(1));

    world
        .commands()
        .entity(target)
        .with_effect((EffectMode::Insert, MyEffect(2)));
    world
        .commands()
        .entity(target)
        .with_effect((EffectMode::Insert, MyEffect(3)));

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&world)
        .map(|c| c.0)
        .collect();

    assert!(effects.contains(&0));
    assert!(effects.contains(&1));
    assert!(!effects.contains(&2));
    assert!(effects.contains(&3));
}

#[test]
fn timer_merge_replace() {
    let mut world = init_world();

    let target = world.spawn_empty().id();
    let second_lifetime = Lifetime::from_seconds(2.0).with_mode(TimerMergeMode::Replace);
    world.commands().entity(target).with_effect((
        EffectMode::Merge,
        Lifetime::from_seconds(1.0).with_mode(TimerMergeMode::Replace),
        MyEffect(0),
    ));
    world.commands().entity(target).with_effect((
        EffectMode::Merge,
        second_lifetime.clone(),
        MyEffect(1),
    ));

    world.flush();

    assert_eq!(
        world.query::<&Lifetime>().single(&world).unwrap(),
        &second_lifetime
    );
    assert_eq!(
        world.query::<&MyEffect>().single(&world).unwrap(),
        &MyEffect(1)
    );
}

#[test]
fn timer_merge_keep() {
    let mut world = init_world();

    let target = world.spawn_empty().id();
    let first_delay = Delay::from_seconds(1.0).with_mode(TimerMergeMode::Keep);
    world.commands().entity(target).with_effect((
        EffectMode::Merge,
        first_delay.clone(),
        MyEffect(0),
    ));
    world.commands().entity(target).with_effect((
        EffectMode::Merge,
        Delay::from_seconds(2.0).with_mode(TimerMergeMode::Keep),
        MyEffect(1),
    ));

    world.flush();

    assert_eq!(
        world.query::<&Delay>().single(&world).unwrap(),
        &first_delay
    );
    assert_eq!(
        world.query::<&MyEffect>().single(&world).unwrap(),
        &MyEffect(1)
    );
}

#[test]
fn timer_merge_fraction() {
    let mut world = init_world();

    let target = world.spawn_empty().id();
    let mut first_timer = Timer::from_seconds(2.0, TimerMode::Once);
    first_timer.tick(Duration::from_secs_f32(1.0));

    world.commands().entity(target).with_effect((
        EffectMode::Merge,
        Delay {
            timer: first_timer,
            mode: TimerMergeMode::Fraction,
        },
        MyEffect(0),
    ));
    world.commands().entity(target).with_effect((
        EffectMode::Merge,
        Delay::from_seconds(10.0).with_mode(TimerMergeMode::Fraction),
        MyEffect(1),
    ));

    world.flush();

    let mut expected_timer = Timer::from_seconds(10.0, TimerMode::Repeating);
    expected_timer.tick(Duration::from_secs_f32(5.0));

    assert_eq!(
        world.query::<&Delay>().single(&world).unwrap(),
        &Delay {
            timer: expected_timer,
            mode: TimerMergeMode::Fraction,
        }
    );
    assert_eq!(
        world.query::<&MyEffect>().single(&world).unwrap(),
        &MyEffect(1)
    );
}

#[test]
fn timer_merge_max() {
    let mut world = init_world();

    let target = world.spawn_empty().id();
    let max = Delay::from_seconds(3.0).with_mode(TimerMergeMode::Max);

    world.commands().entity(target).with_effect((
        EffectMode::Merge,
        Delay::from_seconds(1.0).with_mode(TimerMergeMode::Max),
        MyEffect(0),
    ));
    world
        .commands()
        .entity(target)
        .with_effect((EffectMode::Merge, max.clone(), MyEffect(1)));
    world.commands().entity(target).with_effect((
        EffectMode::Merge,
        Delay::from_seconds(2.0).with_mode(TimerMergeMode::Max),
        MyEffect(2),
    ));

    world.flush();

    assert_eq!(world.query::<&Delay>().single(&world).unwrap(), &max);
    assert_eq!(
        world.query::<&MyEffect>().single(&world).unwrap(),
        &MyEffect(2)
    );
}
