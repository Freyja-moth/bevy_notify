use crate::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use std::marker::PhantomData;

#[derive(Resource)]
struct DetectingRemoved<C: Component>(PhantomData<C>);

#[derive(EntityEvent)]
/// Indicates that the component [`C`] on the monitered entity has been removed.
pub struct Removal<C: Component> {
    pub entity: Entity,
    /// The [`Entity`] that [`C`] was removed from.
    pub removed: Entity,
    pub(crate) _phantom: PhantomData<C>,
}

#[derive(Component)]
#[component(on_add = NotifyRemoved::<C>::register_component_add_observer)]
pub struct NotifyRemoved<C: Component>(PhantomData<C>);
impl<C: Component> Default for NotifyRemoved<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<C: Component> NotifyRemoved<C> {
    fn register_component_add_observer(mut world: DeferredWorld, _: HookContext) {
        if world.contains_resource::<DetectingRemoved<C>>() {
            return;
        }

        let mut commands = world.commands();
        commands.insert_resource(DetectingRemoved::<C>(PhantomData));
        commands.add_observer(notify_on_remove::<C>);
    }
}

pub(crate) fn notify_on_remove<C: Component>(
    remove: On<Remove, C>,
    mut commands: Commands,
    monitors: Populated<(Entity, Option<&Monitoring>), With<NotifyAdded<C>>>,
) {
    monitors
        .iter()
        .filter(|(_, monitoring)| {
            monitoring.is_none_or(|&Monitoring(entity)| entity == remove.entity)
        })
        .for_each(|(entity, _)| {
            commands.trigger(Removal {
                entity,
                removed: remove.entity,
                _phantom: PhantomData::<C>,
            })
        });
}
