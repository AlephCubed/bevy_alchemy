use crate::bundle_inspector::BundleInspector;
use crate::registry::{EffectMergeFn, EffectMergeRegistry};
use crate::{EffectMode, EffectedBy, Effecting};
use bevy_ecs::entity_disabling::Disabled;
use bevy_ecs::prelude::*;
use bevy_log::warn_once;
use std::any::TypeId;

/// Applies an effect to a target entity.
/// This *might* spawn a new entity, depending on what effects are already applied to the target.
///
/// This is normally used via [`with_effect`](EffectCommandsExt::with_effect)
/// or related spawners ([`EffectedBy::spawn`](SpawnRelated::spawn)).
pub struct AddEffectCommand<B: Bundle> {
    /// The entity to apply the effect to.
    pub target: Entity,
    /// The effect to apply.
    pub bundle: B,
}

impl<B: Bundle> AddEffectCommand<B> {
    fn bundle_full(self) -> (Effecting, B) {
        (Effecting(self.target), self.bundle)
    }

    /// Inserts into the existing entity, and then merges the old effect into it using [`EffectMergeRegistry`].
    /// Only registered components that implement `Clone` will be merged.
    /// ## Steps
    /// 1. Copy unregistered components to a new temporary, disabled entity.
    /// 2. Insert new components into the existing entity.
    /// 3. Merge the old components (temp entity) with the new ones (existing entity).
    /// 4. Despawn temp entity.
    fn merge(self, world: &mut World, existing_entity: Entity) {
        if !world.contains_resource::<EffectMergeRegistry>() {
            warn_once!(
                "No `EffectComponentMergeRegistry` found. Did you forget to add the `AlchemyPlugin`?"
            );
            return;
        }

        // Copy existing mergeable components to a temporary entity.
        let new_effect = existing_entity;
        let old_effect = {
            let registry = world.resource::<EffectMergeRegistry>();
            let allow: Vec<TypeId> = registry.merges.keys().copied().collect();

            let temp = world.spawn(Disabled).id();
            world
                .entity_mut(existing_entity)
                .clone_with_opt_in(temp, |builder| {
                    builder.without_required_components(|builder| {
                        builder.allow_by_ids(allow);
                    });
                });

            temp
        };

        world.entity_mut(new_effect).insert(self.bundle);

        // Call merge function on those copied components.
        {
            let old = world.entity(old_effect);
            let archetype = old.archetype();

            let registry = world.resource::<EffectMergeRegistry>();

            let merge_functions: Vec<EffectMergeFn> = archetype
                .components()
                .iter()
                .filter_map(|component_id| {
                    world
                        .components()
                        .get_info(*component_id)
                        .and_then(|info| info.type_id())
                        .and_then(|id| registry.merges.get(&id).copied())
                })
                .collect();

            for merge in merge_functions {
                merge(world.entity_mut(new_effect), old_effect);
            }
        }

        world.despawn(old_effect);
    }
}

impl<B: Bundle + Clone> Command for AddEffectCommand<B> {
    fn apply(self, world: &mut World) {
        let mut inspector = world.get_resource_or_init::<BundleInspector>();
        let (name, mode) = inspector.get_effect_meta(self.bundle.clone());

        if mode == EffectMode::Stack {
            world.spawn(self.bundle_full());
            return;
        }

        let Some(effected_by) = world
            .get::<EffectedBy>(self.target)
            .map(|e| e.collection().clone())
        else {
            world.spawn(self.bundle_full());
            return;
        };

        // Find previous entity that is:
        // 1. effecting the same target,
        // 2. and has the same name (ID).
        let old_entity = effected_by.iter().find_map(|entity| {
            let other_mode = world.get::<EffectMode>(*entity)?;

            // Todo Think more about.
            if mode != *other_mode {
                return None;
            }

            let other_name = world.get::<Name>(*entity);

            if name == other_name.cloned() {
                return Some(*entity);
            }

            None
        });

        let Some(old_entity) = old_entity else {
            world.spawn(self.bundle_full());
            return;
        };

        match mode {
            EffectMode::Stack => unreachable!(),
            EffectMode::Insert => {
                world.entity_mut(old_entity).insert(self.bundle_full());
            }
            EffectMode::Merge => self.merge(world, old_entity),
        }
    }
}

/// Uses commands to apply effects to a specific target entity.
///
/// This is normally used during [`with_effects`](EffectCommandsExt::with_effects).
///
/// # Example
#[doc = include_str!("../docs/with_effects_example.md")]
pub struct EffectSpawner<'a> {
    target: Entity,
    commands: &'a mut Commands<'a, 'a>,
}

impl<'a> EffectSpawner<'a> {
    /// Applies an effect to the target entity.
    /// This *might* spawn a new entity, depending on what effects are already applied to the target.
    ///
    /// This is normally used during [`with_effects`](EffectCommandsExt::with_effects).
    ///
    /// # Example
    #[doc = include_str!("../docs/with_effects_example.md")]
    pub fn spawn<B: Bundle + Clone>(&mut self, bundle: B) {
        self.commands.queue(AddEffectCommand {
            target: self.target,
            bundle,
        });
    }
}

/// An extension trait for adding effect methods to [`EntityCommands`].
pub trait EffectCommandsExt {
    /// Applies an effect to this entity.
    /// This *might* spawn a new entity, depending on what effects are already applied to it.
    ///
    /// For applying multiple effects, see [`with_effects`](Self::with_effects).
    ///
    /// # Example
    #[doc = include_str!("../docs/with_effect_example.md")]
    fn with_effect<B: Bundle + Clone>(&mut self, bundle: B) -> &mut Self;

    /// Applies effects to this entity by taking a function that operates on a [`EffectSpawner`].
    ///
    /// For applying a single effect, see [`with_effect`](Self::with_effect).
    ///
    /// # Example
    #[doc = include_str!("../docs/with_effects_example.md")]
    fn with_effects(&mut self, f: impl FnOnce(&mut EffectSpawner)) -> &mut Self;
}

impl EffectCommandsExt for EntityCommands<'_> {
    fn with_effect<B: Bundle + Clone>(&mut self, bundle: B) -> &mut Self {
        let target = self.id();
        self.commands().queue(AddEffectCommand { target, bundle });
        self
    }

    fn with_effects(&mut self, f: impl FnOnce(&mut EffectSpawner)) -> &mut Self {
        f(&mut EffectSpawner {
            target: self.id(),
            commands: &mut self.commands(),
        });
        self
    }
}
