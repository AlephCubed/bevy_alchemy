use crate::ReflectComponent;
use bevy_ecs::prelude::{Component, Entity};
use bevy_reflect::Reflect;

/// Stores the entity that is being effected by this status effect.
#[derive(Component, Reflect, Eq, PartialEq, Debug, Clone)]
#[relationship(relationship_target = EffectedBy)]
#[reflect(Component, PartialEq, Debug, Clone)]
pub struct Effecting(pub Entity);

/// Stores all the status effects that are effecting this entity.
#[derive(Component, Reflect, Eq, PartialEq, Debug, Clone)]
#[relationship_target(relationship = Effecting, linked_spawn)]
#[reflect(Component, PartialEq, Debug, Clone)]
pub struct EffectedBy(Vec<Entity>);

impl<'a> IntoIterator for &'a EffectedBy {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = std::slice::Iter<'a, Entity>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
