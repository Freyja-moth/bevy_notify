use crate::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use std::marker::PhantomData;

#[derive(Resource)]
/// Used to indicate that the component [`C`] already has an observer detecting when it is added.
struct DetectingAdded<C: Component> {
    observer: Entity,
    _phantom: PhantomData<C>,
}
#[derive(EntityEvent)]
/// Indicates that the component [`C`] on the monitered entity has been added.
pub struct Addition<C: Component> {
    pub entity: Entity,
    /// The [`Entity`] that [`C`] was added to.
    pub added: Entity,
    pub(crate) _phantom: PhantomData<C>,
}

#[derive(Component)]
#[component(
    on_add = NotifyAdded::<C>::register_component_add_observer,
    on_remove = NotifyAdded::<C>::remove_component_add_observer
)]
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
        let observer = commands.add_observer(notify_on_add::<C>).id();
        commands.insert_resource(DetectingAdded::<C> {
            observer,
            _phantom: PhantomData,
        });
    }
    fn remove_component_add_observer(mut world: DeferredWorld, _: HookContext) {
        // # Safety
        // The only component being queried for is on that must already exist in the world for this
        // hook to run
        let total_reactive = world
            .try_query_filtered::<(), With<Self>>()
            .unwrap()
            .iter(&world)
            .count();

        if total_reactive == 0 {
            world.commands().queue(|world: &mut World| {
                // # Safety
                // In order for this component to be removed `NotifyAdded::register_component_add_observer` must have run which adds the `DetectingAdded` resource.
                let DetectingAdded { observer, .. } =
                    world.remove_resource::<DetectingAdded<C>>().unwrap();
                world.entity_mut(observer).despawn();
            });
        }
    }
}

pub(crate) fn notify_on_add<C: Component>(
    add: On<Add, C>,
    mut commands: Commands,
    local_monitors: Query<Entity, (With<NotifyAdded<C>>, With<MonitoringSelf>)>,
    monitors: Query<(Entity, &Monitoring), (With<NotifyAdded<C>>, Without<MonitoringSelf>)>,
    global_monitors: Query<
        Entity,
        (
            With<NotifyAdded<C>>,
            Without<Monitoring>,
            Without<MonitoringSelf>,
        ),
    >,
) {
    if local_monitors.contains(add.entity) {
        commands.trigger(Addition::<C> {
            entity: add.entity,
            added: add.entity,
            _phantom: PhantomData,
        });
    };

    monitors
        .iter()
        .filter(|(_, Monitoring(entity))| *entity == add.entity)
        .for_each(|(entity, &Monitoring(added))| {
            commands.trigger(Addition::<C> {
                entity,
                added,
                _phantom: PhantomData,
            });
        });

    global_monitors.iter().for_each(|entity| {
        commands.trigger(Addition::<C> {
            entity,
            added: add.entity,
            _phantom: PhantomData,
        });
    });
}
