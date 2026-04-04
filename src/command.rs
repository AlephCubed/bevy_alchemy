use crate::bundle_inspector::BundleInspector;
use crate::registry::EffectMergeRegistry;
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
    /// Returns the bundle with the relationship component.
    fn bundle_full(self) -> (Effecting, B) {
        (Effecting(self.target), self.bundle)
    }

    /// Merges the [stashed bundle](Self::stash_bundle) with an entity from the given world.
    /// This is done by calling the [`EffectMergeFn`] for all components in the [registry](EffectMergeRegistry).
    /// Components not in the registry will be copied to the target entity.
    fn merge(self, world: &mut World, existing_entity: Entity) {
        world
            .try_resource_scope::<BundleInspector, ()>(|world, inspector| {
                world.try_resource_scope::<EffectMergeRegistry, ()>(|world, registry| {
                    for incoming_component_id in inspector.get_ref().archetype().components() {
                        let type_id = inspector.get_type_id(*incoming_component_id).unwrap();

                        if let Some(merge) = registry.merges.get(&type_id) {
                            let entity_mut = world.entity_mut(existing_entity);

                            if entity_mut.contains_type_id(type_id) {
                                merge(entity_mut, inspector.get_ref());
                                continue;
                            }
                        }

                        unsafe {
                            // SAFETY: `incoming_component_id` `type_id` were extracted from the inspector.
                            _ = inspector.copy_to_world(world, existing_entity, type_id, *incoming_component_id)
                                .inspect_err(|e| {
                                    warn!("{e}");
                                });
                        }
                    }
                })
                    .or_else(|| {
                        warn_once!("No `EffectMergeRegistry` found. Did you forget to add the `AlchemyPlugin`?");
                        None
                    });
            })
            .or_else(|| {
                warn_once!("No `BundleInspector` found. Did you forget to add the `AlchemyPlugin`?");
                None
            });
    }
}

impl<B: Bundle + Clone> Command for AddEffectCommand<B> {
    fn apply(self, world: &mut World) {
        let mut inspector = world.get_resource_or_init::<BundleInspector>();
        let (name, mode) = inspector
            .stash_bundle(self.bundle.clone())
            .get_effect_meta();

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
            EffectMode::Merge => {
                // Ensure that all components are registered in the main world for cloning into.
                world.register_bundle::<B>();
                self.merge(world, old_entity)
            }
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
