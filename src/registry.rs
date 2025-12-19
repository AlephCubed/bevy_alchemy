use bevy_ecs::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;

pub type EffectMergeFn = fn(world: &mut World, old: Entity, incoming: Entity);

#[derive(Resource, Default)]
pub struct EffectMergeRegistry {
    pub(crate) merges: HashMap<TypeId, EffectMergeFn>,
}

impl EffectMergeRegistry {
    pub fn register<T: Component>(&mut self, f: EffectMergeFn) -> &mut Self {
        self.merges.insert(TypeId::of::<T>(), f);
        self
    }
}
