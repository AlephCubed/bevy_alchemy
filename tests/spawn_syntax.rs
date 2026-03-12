//! Tests the behaviour of the spawn syntax using `SpawnableList`.

use bevy_alchemy::*;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Eq, PartialEq, Default, Clone)]
struct MyEffect(u8);

#[test]
fn spawnable_list_stack() {
    let mut world = World::new();

    world.spawn((
        Name::new("Target"),
        EffectedBy::spawn((EffectBundle(MyEffect(0)), EffectBundle(MyEffect(1)))),
    ));

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
fn spawnable_list_insert() {
    let mut world = World::new();

    world.spawn((
        Name::new("Target"),
        EffectedBy::spawn((
            EffectBundle((EffectMode::Insert, MyEffect(0))),
            EffectBundle((EffectMode::Insert, MyEffect(1))),
        )),
    ));

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
fn spawnable_list_mixed() {
    let mut world = World::new();

    world.spawn((
        Name::new("Target"),
        EffectedBy::spawn((
            EffectBundle(MyEffect(0)),
            EffectBundle(MyEffect(1)),
            EffectBundle((EffectMode::Insert, MyEffect(2))),
            EffectBundle((EffectMode::Insert, MyEffect(3))),
        )),
    ));

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
