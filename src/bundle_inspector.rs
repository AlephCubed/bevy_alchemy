use crate::EffectMode;
use bevy_ecs::component::{ComponentCloneBehavior, ComponentId};
use bevy_ecs::prelude::{
    AppTypeRegistry, Bundle, Entity, EntityRef, Name, ReflectComponent, Resource, World,
};
use bevy_ecs::ptr::OwningPtr;
use bevy_ecs::relationship::RelationshipHookMode;
use bevy_utils::prelude::DebugName;
use std::alloc::alloc;
use std::any::TypeId;
use std::error::Error;
use std::fmt::Formatter;
use std::ptr::{NonNull, copy_nonoverlapping};

#[derive(Resource)]
pub(crate) struct BundleInspector {
    world: World,
    scratch_entity: Entity,
}

impl Default for BundleInspector {
    fn default() -> Self {
        let mut world = World::new();
        let scratch_entity = world.spawn_empty().id();
        Self {
            world,
            scratch_entity,
        }
    }
}

impl BundleInspector {
    /// Stashes a bundle so it can be inspected.
    ///
    /// Calls [`clear`](Self::clear) first to avoid leakage.
    pub fn stash_bundle<B: Bundle>(&mut self, bundle: B) -> &mut Self {
        self.clear();

        self.world
            .entity_mut(self.scratch_entity)
            .insert_with_relationship_hook_mode(bundle, RelationshipHookMode::Skip);

        self
    }

    /// Clears the [stashed bundle](Self::stash_bundle).
    pub fn clear(&mut self) -> &mut Self {
        self.world.entity_mut(self.scratch_entity).clear();

        self
    }

    /// Returns the [stashed bundle's](Self::stash_bundle) name and effect mode.
    pub fn get_effect_meta(&self) -> (Option<Name>, EffectMode) {
        let name = self
            .world
            .entity(self.scratch_entity)
            .get::<Name>()
            .cloned();

        let mode = self
            .world
            .entity(self.scratch_entity)
            .get::<EffectMode>()
            .copied()
            .unwrap_or_default();

        (name, mode)
    }

    /// Returns a reference to the [stashed bundle](Self::stash_bundle).
    pub fn get_ref(&'_ self) -> EntityRef<'_> {
        self.world.entity(self.scratch_entity)
    }

    /// Converts a component ID to a type ID, if registered.
    /// The component ID must be from the inspector's world, using [`get_type_id`](Self::get_type_id).
    pub fn get_type_id(&self, component_id: ComponentId) -> Option<TypeId> {
        self.world
            .components()
            .get_info(component_id)
            .and_then(|info| info.type_id())
    }

    /// Copies a component from the [stashed bundle](Self::stash_bundle) into an entity in a different world.
    /// The component ID must be from the inspector's world, using [`get_type_id`](Self::get_type_id).
    ///
    /// # Errors
    /// Will return an error if:
    /// - The component is not registered in `dst_world`.
    /// - The component cannot be cloned ([`ComponentCloneBehavior::Ignore`]).
    /// - The stashed bundle doesn't contain the component.
    /// - The destination entity doesn't exist in `dst_world`.
    /// - The resource AppTypeRegistry doesn't exist in `dst_world`.
    ///
    /// # Safety
    /// `src_component_id` must be for the same component as `type_id`.
    pub unsafe fn copy_to_world(
        &self,
        dst_world: &mut World,
        dst_entity: Entity,
        type_id: TypeId,
        src_component_id: ComponentId,
    ) -> Result<&Self, MultiWorldCopyError> {
        let Some(dst_component_id) = dst_world.components().get_id(type_id) else {
            return Err(MultiWorldCopyError::Unregistered(type_id));
        };
        let component_info = dst_world.components().get_info(dst_component_id).unwrap();

        match component_info.clone_behavior() {
            ComponentCloneBehavior::Default | ComponentCloneBehavior::Custom(_) => {}
            ComponentCloneBehavior::Ignore => {
                return Err(MultiWorldCopyError::Uncloneable(component_info.name()));
            }
        }

        let Some(src) = self.world.get_by_id(self.scratch_entity, src_component_id) else {
            return Err(MultiWorldCopyError::MissingSrcComponent(
                component_info.name(),
                self.scratch_entity,
            ));
        };

        if component_info.drop().is_none() {
            unsafe {
                // SAFETY: Contract is required to be upheld by the world.
                let dst = alloc(component_info.layout());

                // SAFETY: `dst` is allocated from the component's layout.
                // Both IDs provided by the caller must match, and `src` and `dst` are obtained using those IDs.
                // `src` and `dst` are from different worlds, so cannot overlap.
                copy_nonoverlapping(src.as_ptr(), dst, component_info.layout().size());

                // SAFETY: Both IDs provided by the caller must match, and `dst` was created from `src`.
                let owning = OwningPtr::new(NonNull::new(dst).unwrap());

                // SAFETY: `existing_component_id` is extracted from `dst_world`.
                // Both IDs provided by the caller must match, and `owning` was obtained using `src_component_id`.
                dst_world
                    .get_entity_mut(dst_entity)
                    .map_err(|_| MultiWorldCopyError::MissingDstEntity(dst_entity))?
                    .insert_by_id(dst_component_id, owning);
            }
        } else {
            let Some(registry) = dst_world.get_resource::<AppTypeRegistry>().cloned() else {
                return Err(MultiWorldCopyError::MissingTypeRegistry);
            };
            let registry = registry.read();

            let reflect_component = registry
                .get_type_data::<ReflectComponent>(type_id)
                .ok_or(MultiWorldCopyError::Uncloneable(component_info.name()))?;

            reflect_component.copy(
                &self.world,
                dst_world,
                self.scratch_entity,
                dst_entity,
                &registry,
            );
        }

        Ok(self)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MultiWorldCopyError {
    Unregistered(TypeId),
    Uncloneable(DebugName),
    MissingDstEntity(Entity),
    MissingSrcComponent(DebugName, Entity),
    MissingTypeRegistry,
}

impl std::fmt::Display for MultiWorldCopyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiWorldCopyError::Unregistered(type_id) => write!(
                f,
                "Component with {type_id:?} is not registered in the destination world, and therefor cannot be inserted using merge mode.",
            ),
            MultiWorldCopyError::Uncloneable(name) => write!(
                f,
                "Component {name} cannot be cloned, and therefor cannot be inserted using merge mode.",
            ),
            MultiWorldCopyError::MissingDstEntity(entity) => write!(
                f,
                "Entity {entity} does not exist in the destination world."
            ),
            MultiWorldCopyError::MissingSrcComponent(name, entity) => write!(
                f,
                "Component {name} does not exist on the scratch entity {entity}, and therefor cannot be cloned.",
            ),
            MultiWorldCopyError::MissingTypeRegistry => write!(
                f,
                "Resource AppTypeRegistry does not exist in the destination world, and therefor no components can be cloned.",
            ),
        }
    }
}

impl Error for MultiWorldCopyError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Effecting;

    #[test]
    fn get_effect_meta() {
        let mut inspector = BundleInspector::default();

        let name = Name::new("Effect");
        let mode = EffectMode::Insert;

        assert_eq!(
            inspector
                .stash_bundle((name.clone(), mode))
                .get_effect_meta(),
            (Some(name), mode)
        );
    }

    #[test]
    fn get_effect_meta_no_name() {
        let mut inspector = BundleInspector::default();

        let mode = EffectMode::Insert;

        assert_eq!(inspector.stash_bundle(mode).get_effect_meta(), (None, mode));
    }

    #[test]
    fn get_effect_meta_no_mode() {
        let mut inspector = BundleInspector::default();

        let name = Name::new("Effect");

        assert_eq!(
            inspector.stash_bundle(name.clone()).get_effect_meta(),
            (Some(name), EffectMode::default())
        );
    }

    #[test]
    fn get_effect_meta_nothing() {
        let mut inspector = BundleInspector::default();

        assert_eq!(
            inspector.stash_bundle(()).get_effect_meta(),
            (None, EffectMode::default())
        );
    }

    #[test]
    fn get_effect_mode_with_relation() {
        let mut inspector = BundleInspector::default();

        let name = Name::new("Effect");
        let mode = EffectMode::Insert;

        assert_eq!(
            inspector
                .stash_bundle((
                    name.clone(),
                    mode,
                    Effecting(Entity::from_raw_u32(32).unwrap())
                ))
                .get_effect_meta(),
            (Some(name), mode)
        );
    }
}
