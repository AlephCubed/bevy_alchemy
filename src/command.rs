use crate::bundle_inspector::BundleInspector;
use crate::registry::{EffectMergeFn, EffectMergeRegistry};
use crate::{EffectMode, EffectedBy, Effecting};
use bevy_ecs::prelude::*;
use bevy_log::{warn, warn_once};

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

        world.try_resource_scope::<BundleInspector, ()>(|world, mut inspector| {
            world.try_resource_scope::<EffectMergeRegistry, ()>(|world, registry| {
                let incoming = inspector.get_ref();

                let merge_functions: Vec<EffectMergeFn> = incoming
                    .archetype()
                    .components()
                    .iter()
                    .filter_map(|incoming_component_id| {
                        let type_id = inspector.get_type_id(*incoming_component_id)?;

                        if let Some(merge_fn) = registry.merges.get(&type_id) {
                            return Some(*merge_fn);
                        }

                        _ = unsafe {
                            inspector
                                .copy_to_world(
                                    world,
                                    existing_entity,
                                    type_id,
                                    *incoming_component_id,
                                )
                                .inspect_err(|e| {
                                    warn!("{e}");
                                })
                        };

                        None
                    })
                    .collect();

                let mut existing = world.entity_mut(existing_entity);

                for merge in merge_functions {
                    merge(&mut existing, &incoming);
                }

                inspector.clear();
            });
        });
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

        let Some(effected_by) = world.get::<EffectedBy>(self.target).map(|e| e.collection()) else {
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

            if name.as_ref() == other_name {
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
                world.entity_mut(old_entity).insert(self.bundle);
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

    /// Applies effects to this entity by taking a function that operates on an [`EffectSpawner`].
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
