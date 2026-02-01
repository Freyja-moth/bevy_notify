use crate::prelude::*;
use bevy_app::Update;
use bevy_ecs::{
    lifecycle::HookContext, prelude::*, schedule::ScheduleCleanupPolicy, world::DeferredWorld,
};
use bevy_reflect::Reflect;
use std::marker::PhantomData;

#[derive(Resource, Reflect, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Used to indicate that the component [`C`] is being watched by a system to prevent systems from
/// being added multiple times.
struct DetectingChanges<C>(PhantomData<C>);

#[derive(EntityEvent)]
/// Indicates that the component [`C`] on the monitered entity has changed.
pub struct Mutation<C: Component> {
    pub entity: Entity,
    /// The [`Entity`] that [`C`] belongs to.
    pub mutated: Entity,
    pub(crate) _phantom: PhantomData<C>,
}
#[derive(Component, Reflect, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[component(
    on_add = NotifyChanged::<C>::register_component_change_system, 
    on_remove = NotifyChanged::<C>::remove_component_change_system
)]
/// Specifies that a moniter should react to all changed to [`C`] on the monitered entity.
pub struct NotifyChanged<C: Component>(PhantomData<C>);
impl<C: Component> Default for NotifyChanged<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<C: Component> NotifyChanged<C> {
    fn register_component_change_system(mut world: DeferredWorld, _: HookContext) {
        if world.contains_resource::<DetectingChanges<C>>() {
            return;
        }

        world.commands().queue(|world: &mut World| {
            world.schedule_scope(Update, |_, schedule| {
                schedule.add_systems(watch_for_change::<C>);
            });
            world.insert_resource(DetectingChanges::<C>(PhantomData));
        });
    }
    fn remove_component_change_system(mut world: DeferredWorld, _: HookContext) {
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
                world.schedule_scope(Update, |world, schedule| {
                    // # Safety
                    // This hook can only run when `NotifyChanged::<C>` has been removed which
                    // ensures this sytem must exist in the `Update` schedule.
                    schedule
                        .remove_systems_in_set(
                            watch_for_change::<C>,
                            world,
                            ScheduleCleanupPolicy::RemoveSystemsOnly,
                        )
                        .unwrap();
                });
                world.remove_resource::<DetectingChanges<C>>();
            });
        }
    }
}

fn watch_for_change<C: Component>(
    mut commands: Commands,
    changed: Populated<Entity, Changed<C>>,
    local_monitors: Query<Entity, (With<NotifyChanged<C>>, With<MonitoringSelf>)>,
    monitors: Query<(Entity, &Monitoring), (With<NotifyChanged<C>>, Without<MonitoringSelf>)>,
    global_monitors: Query<
        Entity,
        (
            With<NotifyChanged<C>>,
            Without<MonitoringSelf>,
            Without<Monitoring>,
        ),
    >,
) {
    local_monitors.iter_many(changed.iter()).for_each(|entity| {
        commands.trigger(Mutation::<C> {
            entity,
            mutated: entity,
            _phantom: PhantomData,
        });
    });

    monitors
        .iter()
        .filter(|(_, Monitoring(entity))| changed.contains(*entity))
        .for_each(|(entity, &Monitoring(mutated))| {
            commands.trigger(Mutation::<C> {
                entity,
                mutated,
                _phantom: PhantomData,
            })
        });

    global_monitors.iter().for_each(|global_monitor| {
        changed.iter().for_each(|mutated| {
            commands.trigger(Mutation::<C> {
                entity: global_monitor,
                mutated,
                _phantom: PhantomData,
            });
        });
    });
}
