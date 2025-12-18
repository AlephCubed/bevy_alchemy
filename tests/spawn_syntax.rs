//! Tests the behaviour of the spawn syntax using `SpawnableList`.

use bevy_alchemy::*;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Eq, PartialEq, Default)]
struct MyEffect(u8);

#[test]
fn spawnable_list_stack() {
    let mut world = World::new();

    world.spawn((
        Name::new("Target"),
        EffectedBy::spawn((
            EffectBundle {
                bundle: MyEffect(0),
                ..Default::default()
            },
            EffectBundle {
                bundle: MyEffect(1),
                ..Default::default()
            },
        )),
    ));

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&mut world)
        .map(|c| c.0)
        .collect();

    assert!(effects.contains(&0));
    assert!(effects.contains(&1));
}

#[test]
fn spawnable_list_replace() {
    let mut world = World::new();

    world.spawn((
        Name::new("Target"),
        EffectedBy::spawn((
            EffectBundle {
                mode: EffectMode::Replace,
                bundle: MyEffect(0),
                ..Default::default()
            },
            EffectBundle {
                mode: EffectMode::Replace,
                bundle: MyEffect(1),
                ..Default::default()
            },
        )),
    ));

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&mut world)
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
            EffectBundle {
                bundle: MyEffect(0),
                ..Default::default()
            },
            EffectBundle {
                bundle: MyEffect(1),
                ..Default::default()
            },
            EffectBundle {
                mode: EffectMode::Replace,
                bundle: MyEffect(2),
                ..Default::default()
            },
            EffectBundle {
                mode: EffectMode::Replace,
                bundle: MyEffect(3),
                ..Default::default()
            },
        )),
    ));

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&mut world)
        .map(|c| c.0)
        .collect();

    assert!(effects.contains(&0));
    assert!(effects.contains(&1));
    assert!(!effects.contains(&2));
    assert!(effects.contains(&3));
}
