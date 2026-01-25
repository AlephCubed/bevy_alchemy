use crate::EffectMergeRegistry;
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::ReflectComponent;
use bevy_ecs::prelude::{Component, Entity, EntityWorldMut};
use bevy_reflect::Reflect;
use bevy_reflect::prelude::ReflectDefault;
use std::ops::{Add, AddAssign, Deref, DerefMut};

pub(crate) struct StackPlugin;

impl Plugin for StackPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<EffectMergeRegistry>()
            .register::<EffectStacks>(merge_effect_stacks);
    }
}

/// Tracks the number stacks of a [merge effect](crate::EffectMode::Merge) that have been applied to an entity.
#[derive(Component, Reflect, Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
#[reflect(Component, Default, PartialEq, Debug, Clone)]
pub struct EffectStacks(pub u8);

impl Default for EffectStacks {
    fn default() -> Self {
        Self(1)
    }
}

impl Deref for EffectStacks {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EffectStacks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add<u8> for EffectStacks {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u8> for EffectStacks {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs
    }
}

/// Merge logic for [`EffectStacks`].
fn merge_effect_stacks(mut new: EntityWorldMut, outgoing: Entity) {
    let outgoing = *new.world().get::<EffectStacks>(outgoing).unwrap();
    *new.get_mut::<EffectStacks>().unwrap() += outgoing.0;
}
