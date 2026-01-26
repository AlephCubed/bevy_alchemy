use crate::ReflectComponent;
use crate::registry::EffectMergeRegistry;
use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::component::Mutable;
use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res};
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::world::EntityWorldMut;
use bevy_reflect::Reflect;
use bevy_time::{Time, Timer, TimerMode};
use std::time::Duration;

pub(crate) struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (despawn_finished_lifetimes, tick_delay).chain());
        app.world_mut()
            .get_resource_or_init::<EffectMergeRegistry>()
            .register::<Lifetime>(merge_effect_timer::<Lifetime>)
            .register::<Delay>(merge_effect_timer::<Delay>);
    }
}

/// A [merge function](crate::EffectMergeFn) for [`EffectTimer`] components ([`Lifetime`] and [`Delay`]).
pub fn merge_effect_timer<T: EffectTimer + Component<Mutability = Mutable> + Clone>(
    mut new: EntityWorldMut,
    outgoing: Entity,
) {
    let outgoing = new.world().get::<T>(outgoing).unwrap().clone();
    new.get_mut::<T>().unwrap().merge(&outgoing);
}

/// A [timer](Timer) which is used for status effects and includes a [`TimerMergeMode`].
pub trait EffectTimer: Sized {
    /// Creates a new timer from a duration.
    fn new(duration: Duration) -> Self;

    /// Creates a new time from a duration, in seconds.
    fn from_seconds(seconds: f32) -> Self {
        Self::new(Duration::from_secs_f32(seconds))
    }

    /// A builder that overwrites the current merge mode with a new value.
    fn with_mode(self, mode: TimerMergeMode) -> Self;

    /// Returns reference to the internal timer.
    fn get_timer(&self) -> &Timer;

    /// Returns mutable reference to the internal timer.
    fn get_timer_mut(&mut self) -> &mut Timer;

    /// Returns reference to the timer's merge mode.
    fn get_mode(&self) -> &TimerMergeMode;

    /// Returns mutable reference to the timer's merge mode.
    fn get_mode_mut(&mut self) -> &mut TimerMergeMode;

    /// Merges an old timer (self) with the new one (incoming).
    /// Behaviour depends on the current [`TimerMergeMode`].
    fn merge(&mut self, incoming: &Self) {
        match self.get_mode() {
            TimerMergeMode::Replace => {}
            TimerMergeMode::Keep => *self.get_timer_mut() = incoming.get_timer().clone(),
            TimerMergeMode::Fraction => {
                let fraction = incoming.get_timer().fraction();
                let duration = self.get_timer().duration().as_secs_f32();
                self.get_timer_mut()
                    .set_elapsed(Duration::from_secs_f32(fraction * duration));
            }
            TimerMergeMode::Max => {
                let old = incoming.get_timer().remaining_secs();
                let new = self.get_timer().remaining_secs();

                if old > new {
                    *self.get_timer_mut() = incoming.get_timer().clone();
                }
            }
            TimerMergeMode::Sum => {
                let duration = incoming.get_timer().duration() + self.get_timer().duration();
                self.get_timer_mut().set_duration(duration);
            }
        }
    }
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

            fn get_timer(&self) -> &Timer {
                &self.timer
            }

            fn get_timer_mut(&mut self) -> &mut Timer {
                &mut self.timer
            }

            fn get_mode(&self) -> &TimerMergeMode {
                &self.mode
            }

            fn get_mode_mut(&mut self) -> &mut TimerMergeMode {
                &mut self.mode
            }
        }
    };
}

/// A timer that despawns the effect when the timer finishes.
#[doc(alias = "Duration")]
#[derive(Component, Reflect, Eq, PartialEq, Debug, Clone)]
#[reflect(Component, PartialEq, Debug, Clone)]
pub struct Lifetime {
    /// Tracks the elapsed time. Once the timer is finished, the entity will be despawned.
    pub timer: Timer,
    /// Controls the merge behaviour when an effect is [merged](crate::EffectMode::Merge).
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

/// A repeating timer used for the delay between effect applications.  
#[derive(Component, Reflect, Eq, PartialEq, Debug, Clone)]
#[reflect(Component, PartialEq, Debug, Clone)]
pub struct Delay {
    /// Tracks the elapsed time.
    pub timer: Timer,
    /// Controls the merge behaviour when an effect is [merged](crate::EffectMode::Merge).
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

/// Controls the merge behaviour of a timer when its effect is [merged](crate::EffectMode::Merge).
#[derive(Reflect, Eq, PartialEq, Debug, Copy, Clone)]
#[reflect(PartialEq, Debug, Clone)]
pub enum TimerMergeMode {
    /// The new effect's time will be used, ignoring the old one.
    /// Results in same behaviour as [`EffectMode::Insert`](crate::EffectMode::Insert), but on a per-timer basis.
    Replace,
    /// The old effect's time will be used, ignoring the new one.
    Keep,
    /// The new timer is used, but with the same fraction of the old timer's elapsed time.
    Fraction,
    /// The timer with the larger time remaining will be used.
    Max,
    /// The timers' durations will be added together.
    Sum,
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
