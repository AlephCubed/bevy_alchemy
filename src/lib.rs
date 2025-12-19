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
    /// Multiple of the same effect can exist at once.
    #[default]
    Stack,
    /// When an effect is added, it will replace matching effects.
    Replace,
    /// When an effect is added, it will merge with matching effects.
    /// By default, the incoming effect will replace the old one, same as [`Replace`](Self::Replace).
    /// This can be configured on a per-component basis using the [`EffectMergeRegistry`] resource.
    Merge,
}
