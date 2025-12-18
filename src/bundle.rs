use crate::{Delay, EffectMode, Lifetime};
use bevy_ecs::prelude::*;

/// A "bundle" of components/settings used when applying an effect.
///
/// Note that this doesn't implement [`Bundle`] due to technical limitations.
#[derive(Default)]
pub struct EffectBundle<B: Bundle> {
    /// The name/ID of the effect. Effects with different IDs have no effect on one another.
    pub name: Name,
    /// Describes the logic used when new effect collides with an existing one.
    pub mode: EffectMode,
    /// The duration of the effect.
    #[doc(alias = "duration")]
    pub lifetime: Option<Lifetime>,
    /// Repeating timer used for the delay between effect applications.
    pub delay: Option<Delay>,
    /// Components that will be added to the effect. This is where the actual effect components get added.
    pub bundle: B,
}
