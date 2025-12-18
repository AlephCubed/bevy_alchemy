use crate::ReflectComponent;
use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res};
use bevy_reflect::Reflect;
use bevy_time::{Time, Timer, TimerMode};
use std::time::Duration;

/// A timer which is used for status effects and includes a [`TimerMergeMode`].
pub trait EffectTimer: Sized {
    /// Creates a new timer from a duration.
    fn new(duration: Duration) -> Self;

    /// Creates a new time from a duration, in seconds.
    fn from_seconds(seconds: f32) -> Self {
        Self::new(Duration::from_secs_f32(seconds))
    }

    /// A builder that overwrites the current merge mode with a new value.
    fn with_mode(self, mode: TimerMergeMode) -> Self;

    /// Merges an existing timer (other) with the new one (self).
    /// Behaviour depends on the current [`TimerMergeMode`].
    fn merge(&mut self, other: &Self);
}

macro_rules! impl_effect_timer {
    ($ident:ident, $timer_mode:expr) => {
        impl EffectTimer for $ident {
            fn new(duration: Duration) -> Self {
                Self {
                    timer: Timer::new(duration, $timer_mode),
                    ..Self::default()
                }
            }

            fn with_mode(mut self, mode: TimerMergeMode) -> Self {
                self.mode = mode;
                self
            }

            fn merge(&mut self, other: &Self) {
                match self.mode {
                    TimerMergeMode::Replace => {}
                    TimerMergeMode::Inherit => self.timer = other.timer.clone(),
                    TimerMergeMode::Fraction => {
                        let fraction = other.timer.fraction();
                        let duration = self.timer.duration().as_secs_f32();
                        self.timer
                            .set_elapsed(Duration::from_secs_f32(fraction * duration))
                    }
                    TimerMergeMode::Max => {
                        let old = other.timer.remaining_secs();
                        let new = self.timer.remaining_secs();

                        if old > new {
                            self.timer = other.timer.clone()
                        }
                    }
                }
            }
        }
    };
}

/// Despawns the entity when the timer finishes.
#[doc(alias = "Duration")]
#[derive(Component, Reflect, Eq, PartialEq, Debug, Clone)]
#[reflect(Component, PartialEq, Debug, Clone)]
pub struct Lifetime {
    /// Tracks the elapsed time. Once the timer is finished, the entity will be despawned.
    pub timer: Timer,
    /// Controls the merge behaviour when an effect is [replaced](EffectMode::Replace).
    pub mode: TimerMergeMode,
}

impl_effect_timer!(Lifetime, TimerMode::Once);

impl Default for Lifetime {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
            mode: TimerMergeMode::Max,
        }
    }
}

/// Repeating timer used for the delay between effect applications.  
#[derive(Component, Reflect, Eq, PartialEq, Debug, Clone)]
#[reflect(Component, PartialEq, Debug, Clone)]
pub struct Delay {
    /// Tracks the elapsed time.
    pub timer: Timer,
    /// Controls the merge behaviour when an effect is [replaced](EffectMode::Replace).
    pub mode: TimerMergeMode,
}

impl_effect_timer!(Delay, TimerMode::Repeating);

impl Default for Delay {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
            mode: TimerMergeMode::Fraction,
        }
    }
}

/// Controls the merge behaviour of a timer when it's effect is [merged](EffectMode::Merge).
#[derive(Reflect, Eq, PartialEq, Debug, Copy, Clone)]
#[reflect(PartialEq, Debug, Clone)]
pub enum TimerMergeMode {
    /// The new effect's time will be used, ignoring the old one.
    /// Same as [`EffectMode::Replace`], but on a per-timer basis.
    Replace,
    /// The old effect's time will be used, ignoring the new one.
    Inherit,
    /// The new timer is used, but with the same fraction of the old timer's elapsed time.
    Fraction,
    /// The timer with the larger time remaining will be used.
    Max,
    // Todo Sum.
}

pub(super) fn despawn_finished_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn tick_delay(time: Res<Time>, mut query: Query<&mut Delay>) {
    for mut delay in &mut query {
        delay.timer.tick(time.delta());
    }
}
