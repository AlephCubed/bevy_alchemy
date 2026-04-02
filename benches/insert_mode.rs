//! Benchmarks for applying insert-mode effects.

use bevy_alchemy::{AlchemyPlugin, Effect, EffectCommandsExt, EffectMode, EffectedBy};
use bevy_app::App;
use bevy_ecs::name::Name;
use bevy_ecs::prelude::{Component, Entity, SpawnRelated};
use criterion::{Criterion, criterion_group, criterion_main};

#[derive(Component, Clone)]
struct BenchEffect;

fn init_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(AlchemyPlugin);

    let entity = app
        .world_mut()
        .spawn((
            Name::new("Target"),
            EffectedBy::spawn(Effect((
                Name::new("Effect"),
                EffectMode::Insert,
                BenchEffect,
            ))),
        ))
        .id();

    (app, entity)
}

fn with_effect(c: &mut Criterion) {
    let (mut app, entity) = init_app();

    c.bench_function("Insert mode matched `with_effect`", |b| {
        b.iter(|| {
            app.world_mut().commands().entity(entity).with_effect((
                Name::new("Effect"),
                EffectMode::Insert,
                BenchEffect,
            ));
            app.world_mut().flush();
        })
    });
}

fn related_spawner(c: &mut Criterion) {
    let (mut app, entity) = init_app();

    c.bench_function("Insert mode matched related spawner", |b| {
        b.iter(|| {
            app.world_mut()
                .commands()
                .entity(entity)
                .insert(EffectedBy::spawn(Effect((
                    Name::new("Effect"),
                    EffectMode::Insert,
                    BenchEffect,
                ))));
            app.world_mut().flush();
        })
    });
}

criterion_group!(benches, with_effect, related_spawner);
criterion_main!(benches);
