use bevy_ecs::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;

/// A function used to merge effects with [`EffectMode::Merge`](crate::EffectMode::Merge), which must be registered in the [registry](EffectMergeRegistry).
pub type EffectMergeFn = fn(world: &mut World, old: Entity, incoming: Entity);

/// Stores the effect merge logic for each registered component.
/// New components can be registered by providing a [`EffectMergeFn`] to the [`register`](EffectMergeRegistry::register) method.
#[derive(Resource, Default)]
pub struct EffectMergeRegistry {
    pub(crate) merges: HashMap<TypeId, EffectMergeFn>,
}

impl EffectMergeRegistry {
    /// Registers a [`EffectMergeFn`] to be run whenever two `T` status effects are merged.
    pub fn register<T: Component>(&mut self, f: EffectMergeFn) -> &mut Self {
        self.merges.insert(TypeId::of::<T>(), f);
        self
    }
}
