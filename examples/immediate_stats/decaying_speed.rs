//! This example shows using [Immediate Stats](https://github.com/AlephCubed/immediate_stats)
//! to add a decaying movement speed buff.
//! This means that the strength of the buff decreases throughout its duration.
//!
//! This uses [`EffectMode::Merge`], which prevents having multiple of the effect applied at the
//! same time (no 10x speed multiplier for you).
//!
//! There is a second version of this example, which uses Bevy Auto Plugin.

use bevy::prelude::*;
use bevy_alchemy::*;
use immediate_stats::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AlchemyPlugin, ImmediateStatsPlugin))
        .add_plugins(ResetComponentPlugin::<MovementSpeed>::new())
        .add_systems(Startup, init_scene)
        .add_systems(Update, (on_space_pressed, apply_speed_boost))
        .add_systems(PostUpdate, update_ui)
        .run();
}

/// Tracks an entities current movement speed.
#[derive(Component, StatContainer)]
struct MovementSpeed(Stat);

/// Applies a speed boost, which decreases throughout its duration.
#[derive(Component, Default)]
struct DecayingSpeed {
    start_speed_boost: Modifier,
}

/// Spawn a target on startup.
fn init_scene(mut commands: Commands) {
    commands.spawn((Name::new("Target"), MovementSpeed(Stat::new(100))));
    commands.spawn(Text::default());
    commands.spawn(Camera2d);
}

/// When space is pressed, apply decaying speed to the target.
fn on_space_pressed(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    target: Single<Entity, With<MovementSpeed>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    commands.entity(*target).with_effect(EffectBundle {
        mode: EffectMode::Insert, // Block having multiple of effect stacked on a single target.
        bundle: (
            Lifetime::from_seconds(2.0), // The duration of the effect.
            DecayingSpeed {
                start_speed_boost: Modifier {
                    bonus: 10,
                    multiplier: 2.0,
                },
            },
        ),
        ..default()
    });
}

/// Applies the effect to the target. Because of how Immediate Stats works, this needs to run every frame.
fn apply_speed_boost(
    effects: Query<(&Effecting, &Lifetime, &DecayingSpeed)>,
    mut targets: Query<&mut MovementSpeed>,
) {
    for (target, lifetime, effect) in effects {
        // Skip if the target doesn't have movement speed.
        let Ok(mut speed) = targets.get_mut(target.0) else {
            continue;
        };

        // Otherwise, apply the buff, scaled by the remaining time.
        speed.0.apply_scaled(
            effect.start_speed_boost,
            lifetime.timer.fraction_remaining(),
        );
    }
}

/// Updates the UI to match the world state.
fn update_ui(
    mut ui: Single<&mut Text>,
    target: Single<&MovementSpeed>,
    effects: Query<(Entity, &Lifetime, &DecayingSpeed)>,
) {
    ui.0 = "Press Space to apply decaying movement speed\n\n".to_string();

    ui.0 += &format!("Speed: {:.1} ({:.1})\n\n", target.0.total(), target.0);

    for (entity, lifetime, speed) in &effects {
        ui.0 += &format!(
            "{} - {:.1}s ({:.1})\n",
            entity,
            lifetime.timer.remaining_secs(),
            speed
                .start_speed_boost
                .scaled(lifetime.timer.fraction_remaining())
        );
    }
}
