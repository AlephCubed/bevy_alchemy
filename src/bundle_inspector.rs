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
