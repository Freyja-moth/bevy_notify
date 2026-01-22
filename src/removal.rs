use crate::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_reflect::Reflect;
use std::marker::PhantomData;

#[derive(Resource, Reflect, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct DetectingRemoved<C: Component>(PhantomData<C>);

#[derive(EntityEvent)]
/// Indicates that the component [`C`] on the monitered entity has been removed.
pub struct Removal<C: Component> {
    pub entity: Entity,
    /// The [`Entity`] that [`C`] was removed from.
    pub removed: Entity,
    pub(crate) _phantom: PhantomData<C>,
}

#[derive(Component, Reflect, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[component(on_add = NotifyAdded::<C>::register_component_add_observer)]
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
    local_monitors: Query<Entity, (With<NotifyRemoved<C>>, With<MonitoringSelf>)>,
    monitors: Query<(Entity, &Monitoring), (With<NotifyRemoved<C>>, Without<MonitoringSelf>)>,
    global_monitors: Query<
        Entity,
        (
            With<NotifyRemoved<C>>,
            Without<Monitoring>,
            Without<MonitoringSelf>,
        ),
    >,
) {
    if local_monitors.contains(remove.entity) {
        commands.trigger(Removal::<C> {
            entity: remove.entity,
            removed: remove.entity,
            _phantom: PhantomData,
        });
    };

    monitors
        .iter()
        .filter(|(_, Monitoring(entity))| *entity == remove.entity)
        .for_each(|(entity, &Monitoring(removed))| {
            commands.trigger(Removal::<C> {
                entity,
                removed,
                _phantom: PhantomData,
            });
        });

    global_monitors.iter().for_each(|entity| {
        commands.trigger(Removal::<C> {
            entity,
            removed: remove.entity,
            _phantom: PhantomData,
        });
    });
}
