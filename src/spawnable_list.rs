use crate::{AddEffectCommand, Effecting};
use bevy_ecs::prelude::*;
use bevy_ecs::ptr::MovingPtr;
use bevy_ecs::spawn::SpawnableList;

/// A wrapper over a [`Bundle`] indicating that an effect should be applied with that [`Bundle`].
/// This is intended to be used in [`EffectedBy::spawn`](SpawnRelated::spawn).
///
/// This *might* spawn a new entity, depending on what effects are already applied to the target.
/// # Example
#[doc = include_str!("../docs/effected_by_spawn_example.md")]
#[derive(Default)]
pub struct Effect<B: Bundle>(pub B);

// Todo This is probably bad practice/has larger performance cost.
impl<B: Bundle + Clone> SpawnableList<Effecting> for Effect<B> {
    fn spawn(this: MovingPtr<'_, Self>, world: &mut World, target: Entity) {
        let bundle = this.read();
        world.commands().queue(AddEffectCommand {
            target,
            bundle: bundle.0,
        });
    }

    fn size_hint(&self) -> usize {
        0
    }
}
