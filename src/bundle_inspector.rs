use crate::EffectMode;
use bevy_ecs::prelude::{Bundle, Entity, Name, Resource, World};
use bevy_ecs::relationship::RelationshipHookMode;

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
    pub fn get_effect_meta<B: Bundle>(&mut self, bundle: B) -> (Option<Name>, EffectMode) {
        let e = self.scratch_entity;
        self.world
            .entity_mut(e)
            .insert_with_relationship_hook_mode(bundle, RelationshipHookMode::Skip);

        let name = self.world.entity(e).get::<Name>().cloned();

        let mode = self
            .world
            .entity_mut(e)
            .get::<EffectMode>()
            .copied()
            .unwrap_or_default();

        self.world.entity_mut(e).clear();

        (name, mode)
    }
}

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
            inspector.get_effect_meta((name.clone(), mode)),
            (Some(name), mode)
        );
    }

    #[test]
    fn get_effect_meta_no_name() {
        let mut inspector = BundleInspector::default();

        let mode = EffectMode::Insert;

        assert_eq!(inspector.get_effect_meta(mode), (None, mode));
    }

    #[test]
    fn get_effect_meta_no_mode() {
        let mut inspector = BundleInspector::default();

        let name = Name::new("Effect");

        assert_eq!(
            inspector.get_effect_meta(name.clone()),
            (Some(name), EffectMode::default())
        );
    }

    #[test]
    fn get_effect_meta_nothing() {
        let mut inspector = BundleInspector::default();

        assert_eq!(inspector.get_effect_meta(()), (None, EffectMode::default()));
    }

    #[test]
    fn get_effect_mode_with_relation() {
        let mut inspector = BundleInspector::default();

        let name = Name::new("Effect");
        let mode = EffectMode::Insert;

        assert_eq!(
            inspector.get_effect_meta((
                name.clone(),
                mode,
                Effecting(Entity::from_raw_u32(32).unwrap())
            )),
            (Some(name), mode)
        );
    }
}
