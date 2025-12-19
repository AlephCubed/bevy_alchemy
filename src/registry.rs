use bevy_ecs::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;

/// A function used to merge effects with [`EffectMode::Merge`](crate::EffectMode::Merge),
/// which must be registered in the [registry](EffectMergeRegistry).
///
/// # Example
/// ```rust
/// # use bevy_ecs::prelude::*;
/// # use bevy_alchemy::EffectMergeRegistry;
/// #[derive(Component, Clone)]
/// struct MyEffect(f32);
///
/// fn merge_my_effect(world: &mut World, old: Entity, incoming: Entity) {
///     let incoming = world.get::<MyEffect>(incoming).unwrap().clone();
///     let mut old = world.get_mut::<MyEffect>(old).unwrap();
///     old.0 + incoming.0;
/// }
/// ```
pub type EffectMergeFn = fn(world: &mut World, old: Entity, incoming: Entity);

/// Stores the effect merge logic for each registered component.
/// New components can be registered by providing a [`EffectMergeFn`] to the [`register`](EffectMergeRegistry::register) method.
/// This function will be run whenever an effect is applied twice to the same entity with [`EffectMode::Merge`](crate::EffectMode::Merge).
///
/// # Example
/// ```rust
/// # use bevy_ecs::prelude::*;
/// # use bevy_alchemy::EffectMergeRegistry;
/// #[derive(Component, Clone)]
/// struct MyEffect(f32);
///
/// fn main() {
///     let mut world = World::new();
///     world.init_resource::<EffectMergeRegistry>();
///
///     world.resource_mut::<EffectMergeRegistry>()
///         .register::<MyEffect>(merge_my_effect);
/// }
///
/// fn merge_my_effect(world: &mut World, old: Entity, incoming: Entity) {
///     let incoming = world.get::<MyEffect>(incoming).unwrap().clone();
///     let mut old = world.get_mut::<MyEffect>(old).unwrap();
///     old.0 + incoming.0;
/// }
/// ```
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
