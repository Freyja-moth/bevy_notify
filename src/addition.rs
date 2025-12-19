use crate::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use std::marker::PhantomData;

#[derive(Resource)]
/// Used to indicate that the component [`C`] already has an observer detecting when it is added.
struct DetectingAdded<C: Component>(PhantomData<C>);

#[derive(EntityEvent)]
/// Indicates that the component [`C`] on the monitered entity has been added.
pub struct Addition<C: Component> {
    pub entity: Entity,
    /// The [`Entity`] that [`C`] was added to.
    pub added: Entity,
    pub(crate) _phantom: PhantomData<C>,
}

#[derive(Component)]
#[component(on_add = NotifyAdded::<C>::register_component_add_observer)]
pub struct NotifyAdded<C: Component>(PhantomData<C>);
impl<C: Component> Default for NotifyAdded<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<C: Component> NotifyAdded<C> {
    fn register_component_add_observer(mut world: DeferredWorld, _: HookContext) {
        if world.contains_resource::<DetectingAdded<C>>() {
            return;
        }

        let mut commands = world.commands();
        commands.insert_resource(DetectingAdded::<C>(PhantomData));
        commands.add_observer(notify_on_add::<C>);
    }
}

pub(crate) fn notify_on_add<C: Component>(
    add: On<Add, C>,
    mut commands: Commands,
    internal_monitors: Query<(), (With<MonitoringSelf>, With<NotifyAdded<C>>)>,
    monitors: Query<(Entity, Option<&Monitoring>), (With<NotifyAdded<C>>, Without<MonitoringSelf>)>,
) {
    if internal_monitors.contains(add.entity) {
        commands.trigger(Removal {
            entity: add.entity,
            removed: add.entity,
            _phantom: PhantomData::<C>,
        });
    }

    monitors
        .iter()
        .filter(|(_, monitoring)| monitoring.is_none_or(|&Monitoring(entity)| entity == add.entity))
        .for_each(|(entity, _)| {
            commands.trigger(Addition {
                entity,
                added: add.entity,
                _phantom: PhantomData::<C>,
            })
        });
}
