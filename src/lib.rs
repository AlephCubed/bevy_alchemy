#![doc = include_str!("../README.md")]

mod bundle;
mod command;
mod registry;
mod relation;
mod timer;

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_reflect::prelude::ReflectDefault;

pub use bundle::*;
pub use command::*;
pub use registry::*;
pub use relation::*;
pub use timer::*;

/// Setup required types and systems for `bevy_alchemy`.
pub struct AlchemyPlugin;

impl Plugin for AlchemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectMode>()
            .register_type::<Effecting>()
            .register_type::<EffectedBy>()
            .register_type::<Lifetime>()
            .register_type::<Delay>()
            .register_type::<TimerMergeMode>()
            .init_resource::<EffectMergeRegistry>()
            .add_plugins(TimerPlugin);
    }
}

/// Describes the logic used when multiple of the same effect are applied to an entity.
#[derive(Component, Reflect, Eq, PartialEq, Debug, Default, Copy, Clone)]
#[reflect(Component, PartialEq, Debug, Default, Clone)]
pub enum EffectMode {
    /// Multiple of the same effect can exist at once, so when an effect is added, its components will be spawned as a new entity.
    #[default]
    Stack,
    /// When an effect is added, its components will be inserted into a matching effect.
    /// If there are no matches, the components will be spawned as a new entity.
    Insert,
    /// When an effect is added, its components will be merged into any matching effect.
    /// By default, the incoming component will replace the old one, same as [`Insert`](Self::Insert).
    /// This can be configured on a per-component basis using the [`EffectMergeRegistry`] resource.
    ///
    /// If there are no matching effects, the components will be spawned as a new entity.
    Merge,
}
