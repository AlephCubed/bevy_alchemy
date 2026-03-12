use bevy_ecs::prelude::*;

/// A "bundle" of components/settings used when applying an effect.
/// Due to technical limitations, this doesn't actually implement [`Bundle`].
/// Instead, purpose build commands ([`with_effect`](crate::command::EffectCommandsExt::with_effect))
/// or related spawners ([`EffectedBy::spawn`](SpawnRelated::spawn)) should be used.
///
/// # Examples
/// ### [`with_effect`](crate::command::EffectCommandsExt::with_effect)
#[doc = include_str!("../docs/with_effect_example.md")]
/// ### [`with_effects`](crate::command::EffectCommandsExt::with_effects) + [`EffectSpawner`](crate::command::EffectSpawner)
#[doc = include_str!("../docs/with_effects_example.md")]
/// ### [`EffectedBy::spawn`](SpawnRelated::spawn)
#[doc = include_str!("../docs/effected_by_spawn_example.md")]
#[derive(Default)]
pub struct EffectBundle<B: Bundle>(pub B);
