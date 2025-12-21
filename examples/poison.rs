//! A simple damage-over-time effect.
//! Todo Add UI.

use bevy::prelude::*;
use bevy_alchemy::{
    AlchemyPlugin, Delay, EffectBundle, EffectCommandsExt, EffectTimer, Effecting, Lifetime,
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
    commands.spawn((Name::new("Target"), Health(100)));
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
        lifetime: Some(Lifetime::from_seconds(4.0)), // The duration of the effect.
        delay: Some(Delay::from_seconds(1.0)),       // The time between damage ticks.
        bundle: Poison { damage: 1 },                // The amount of damage to apply per tick.
        ..default()
    });
}

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

fn update_ui(
    mut ui: Single<&mut Text>,
    target: Single<&Health>,
    effects: Query<(Entity, &Lifetime, &Delay), With<Poison>>,
) {
    ui.0 = "Press Space to apply poison\n\n".to_string();

    ui.0 += &format!("Health: {}\n\n", target.0);

    for (entity, lifetime, delay) in &effects {
        ui.0 += &format!(
            "{} - {:.1}s (tick in {:.1}s)\n",
            entity,
            lifetime.timer.remaining_secs(),
            delay.timer.remaining_secs()
        );
    }
}
