//! Benchmarks for standard Bevy equivalents of effect operations.

use bevy_alchemy::{AlchemyPlugin, EffectMode, EffectedBy, Effecting};
use bevy_app::App;
use bevy_ecs::name::Name;
use bevy_ecs::prelude::{Component, Entity, Spawn};
use bevy_ecs::spawn::SpawnRelated;
use criterion::{Criterion, criterion_group, criterion_main};

#[derive(Component)]
struct BenchEffect;

fn init_app() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(AlchemyPlugin);

    let target = app.world_mut().spawn(Name::new("Target")).id();
    let effect = app
        .world_mut()
        .spawn((
            Effecting(target),
            Name::new("Effect"),
            EffectMode::Stack,
            BenchEffect,
        ))
        .id();

    (app, target, effect)
}

/// Spawning new effect, similar to `EffectMode::Stack`.
fn with_related(c: &mut Criterion) {
    let (mut app, target, _) = init_app();

    c.bench_function("Baseline `with_related`", |b| {
        b.iter(|| {
            app.world_mut()
                .commands()
                .entity(target)
                .with_related::<Effecting>((Name::new("Effect"), EffectMode::Stack, BenchEffect));
            app.world_mut().flush();
        })
    });
}

/// Spawning new effect, similar to `EffectMode::Stack`.
fn related_spawner(c: &mut Criterion) {
    let (mut app, target, _) = init_app();

    c.bench_function("Baseline related spawner", |b| {
        b.iter(|| {
            app.world_mut()
                .commands()
                .entity(target)
                .insert(EffectedBy::spawn(Spawn((
                    Name::new("Effect"),
                    EffectMode::Stack,
                    BenchEffect,
                ))));
            app.world_mut().flush();
        })
    });
}

/// Inserting into an existing effect, similar to `EffectMode::Insert`.
fn insert(c: &mut Criterion) {
    let (mut app, _, effect) = init_app();

    c.bench_function("Baseline insert", |b| {
        b.iter(|| {
            app.world_mut().commands().entity(effect).insert((
                Name::new("Effect"),
                EffectMode::Stack,
                BenchEffect,
            ));
            app.world_mut().flush();
        })
    });
}

criterion_group!(benches, with_related, related_spawner, insert);
criterion_main!(benches);
