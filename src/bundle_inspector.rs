use crate::EffectMode;
use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::{Bundle, Entity, EntityRef, Name, Resource, World};
use bevy_ecs::ptr::OwningPtr;
use bevy_ecs::relationship::RelationshipHookMode;
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
    pub fn stash_bundle<B: Bundle>(&mut self, bundle: B) -> &mut Self {
        self.world
            .entity_mut(self.scratch_entity)
            .insert_with_relationship_hook_mode(bundle, RelationshipHookMode::Skip);

        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.world.entity_mut(self.scratch_entity).clear();

        self
    }

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

    pub fn get_ref(&'_ self) -> EntityRef<'_> {
        self.world.entity(self.scratch_entity)
    }

    pub fn get_type_id(&self, component_id: ComponentId) -> Option<TypeId> {
        self.world
            .components()
            .get_info(component_id)
            .and_then(|info| info.type_id())
    }

    pub unsafe fn copy_to_world(
        &self,
        dst_world: &mut World,
        dst_entity: Entity,
        type_id: TypeId,
        src_component_id: ComponentId,
    ) -> Result<&Self, MultiWorldCopyError> {
        let Some(existing_component_id) = dst_world.components().get_id(type_id) else {
            return Err(MultiWorldCopyError::Unregistered(type_id));
        };

        let component_info = dst_world
            .components()
            .get_info(existing_component_id)
            .unwrap();

        if component_info.drop().is_some() {
            return Err(MultiWorldCopyError::UnCopyable(type_id));
        }

        let Some(src) = self.world.get_by_id(self.scratch_entity, src_component_id) else {
            return Err(MultiWorldCopyError::MissingSrcComponent(type_id));
        };

        unsafe {
            // SAFETY: Contract is required to be upheld by the world.
            let dst = alloc(component_info.layout());

            copy_nonoverlapping(src.as_ptr(), dst, component_info.layout().size());

            let owning = OwningPtr::new(NonNull::new(dst).unwrap());

            dst_world
                .get_entity_mut(dst_entity)
                .map_err(|_| MultiWorldCopyError::MissingDstEntity(dst_entity))?
                .insert_by_id(existing_component_id, owning);
        }

        Ok(self)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MultiWorldCopyError {
    Unregistered(TypeId),
    UnCopyable(TypeId),
    MissingDstEntity(Entity),
    MissingSrcComponent(TypeId),
}

impl std::fmt::Display for MultiWorldCopyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiWorldCopyError::Unregistered(type_id) => write!(
                f,
                "Component with type ID {type_id:?} has not been registered in the inspector world, and therefor cannot be inserted using merge mode."
            ),
            MultiWorldCopyError::UnCopyable(type_id) => write!(
                f,
                "Component with type ID {type_id:?} cannot be copied, and therefor cannot be inserted using merge mode."
            ),
            MultiWorldCopyError::MissingDstEntity(entity) => write!(
                f,
                "Entity {entity} does not exist in the destination world."
            ),
            MultiWorldCopyError::MissingSrcComponent(type_id) => write!(
                f,
                "Component with type ID {type_id:?} does not exist in inspector world, and therefor cannot be inserted using merge mode."
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
