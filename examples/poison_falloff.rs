//! A damage-over-time effect where the damage falls off as more stacks are added.
//!
//! When an entity is already poisoned, subsequent applications deal less damage.
//! In this case the first stack deals 5 damage, the next 4, then 3, and so on.
//!
//! This works by [merging](EffectMode::Merge) the effects into a single entity and using the
//! [number of stacks](EffectStacks) in damage calculations.
//! A slightly simpler version is available in the `poison` example.

use bevy::prelude::*;
use bevy_alchemy::{
    AlchemyPlugin, Delay, EffectBundle, EffectCommandsExt, EffectMode, EffectStacks, EffectTimer,
    Effecting, Lifetime,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AlchemyPlugin))
        .add_systems(Startup, init_scene)
        .add_systems(Update, (on_space_pressed, deal_poison_damage))
        .add_systems(PostUpdate, update_ui)
        .run();
}

#[derive(Component)]
struct Health(i32);

/// Deals damage over time to the target entity.
#[derive(Component, Default)]
struct Poison {
    damage: i32,
}

/// Spawn a target on startup.
fn init_scene(mut commands: Commands) {
    commands.spawn((Name::new("Target"), Health(500)));
    commands.spawn(Text::default());
    commands.spawn(Camera2d);
}

/// When space is pressed, apply poison to the target.
fn on_space_pressed(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    target: Single<Entity, With<Health>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    commands.entity(*target).with_effect(EffectBundle {
        mode: EffectMode::Merge, // Stack tracking requires effect merging.
        bundle: (
            EffectStacks::default(),     // Enable stack tracking.
            Lifetime::from_seconds(4.0), // The duration of the effect.
            Delay::from_seconds(1.0),    // The time between damage ticks.
            Poison { damage: 5 },        // The amount of damage to apply per tick.
        ),
        ..default()
    });
}

/// Runs every frame and deals the poison damage.
fn deal_poison_damage(
    effects: Query<(&Effecting, &EffectStacks, &Delay, &Poison)>,
    mut targets: Query<&mut Health>,
) {
    for (target, stacks, delay, poison) in effects {
        // We wait until the delay finishes to apply the damage.
        if !delay.timer.is_finished() {
            continue;
        }

        // Skip if the target doesn't have health.
        let Ok(mut health) = targets.get_mut(target.0) else {
            continue;
        };

        // Otherwise, deal the damage scaled with the number of stacks.
        // Each subsequent stack has a decreasing effect, the first deals 5 damage, the next 4, then 3, and so on.
        let stacks = poison.damage.min(stacks.0 as i32); // Clamp stacks to prevent negative damage.
        let sub = (stacks * (stacks - 1)) / 2;
        let damage = poison.damage * stacks - sub;

        info!("Dealt {damage} damage!");

        health.0 -= damage;
    }
}

fn update_ui(
    mut ui: Single<&mut Text>,
    target: Single<&Health>,
    effects: Query<(Entity, &EffectStacks, &Lifetime, &Delay), With<Poison>>,
) {
    ui.0 = "Press Space to apply poison\n\n".to_string();

    ui.0 += &format!("Health: {}\n\n", target.0);

    for (entity, stacks, lifetime, delay) in &effects {
        ui.0 += &format!(
            "{}, {} stacks - {:.1}s (tick in {:.1}s)\n",
            entity,
            stacks.0,
            lifetime.timer.remaining_secs(),
            delay.timer.remaining_secs()
        );
    }
}
