use crate::prelude::*;
use bevy_app::Update;
use bevy_ecs::{
    lifecycle::HookContext, prelude::*, schedule::ScheduleCleanupPolicy, world::DeferredWorld,
};
use bevy_reflect::Reflect;
use std::marker::PhantomData;

#[derive(SystemSet, Hash, PartialEq, Eq, Clone, Debug, Default)]
/// The set that triggers reactivity for [`Mutation`]
pub struct MutationSet;

#[derive(Resource, Reflect, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Used to indicate that the component [`C`] is being watched by a system to prevent systems from
/// being added multiple times.
struct DetectingChanges<C>(PhantomData<C>);
impl<C> Default for DetectingChanges<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(EntityEvent)]
/// Indicates that the component [`C`] has been changed on an entity watched by a monitor.
///
/// See [`NotifyChanged<C>`] for more information on how this is triggered.
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
/// Adding this component to a entity will cause it to react to changes on component [`C`] with
/// [`Mutation<C>`]
///
/// By default this will react to changes on **all** entities. See [`Monitor`], and [`MonitorSelf`]
/// for restricting this.
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
                schedule.configure_sets(MutationSet);
                schedule.add_systems(watch_for_change::<C>.in_set(MutationSet));
            });
            world.insert_resource(DetectingChanges::<C>::default());
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
    local_monitors: Query<Entity, (With<NotifyChanged<C>>, With<MonitorSelf>)>,
    monitors: Query<(Entity, &Monitor), With<NotifyChanged<C>>>,
    global_monitors: Query<
        Entity,
        (
            With<NotifyChanged<C>>,
            Without<MonitorSelf>,
            Without<Monitor>,
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
        .filter(|(_, Monitor(entity))| changed.contains(*entity))
        .for_each(|(entity, &Monitor(mutated))| {
            commands.trigger(Mutation::<C> {
                entity,
                mutated,
                _phantom: PhantomData,
            });
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct Player;

    #[test]
    fn check_for_mutation() {
        #[derive(Resource, Default, Debug)]
        pub struct TimesMoved(usize);

        #[derive(EntityEvent)]
        pub struct MoveX {
            #[event_target]
            actor: Entity,
            amount: f32,
        }

        let mut world = World::new();

        world.add_schedule(Schedule::new(Update));

        world.init_resource::<TimesMoved>();

        let player = world
            .spawn((
                Player,
                Transform::default(),
                MonitorSelf,
                NotifyChanged::<Transform>::default(),
            ))
            .observe(
                |_: On<Mutation<Transform>>, mut times_moved: ResMut<TimesMoved>| {
                    times_moved.0 += 1;
                },
            )
            .observe(
                |move_x: On<MoveX>,
                 mut transforms: Query<&mut Transform>|
                 -> Result<(), BevyError> {
                    let mut transform = transforms.get_mut(move_x.actor)?;

                    transform.translation.x += move_x.amount;
                    Ok(())
                },
            )
            .id();

        assert_eq!(world.resource::<TimesMoved>().0, 0);

        world.trigger(MoveX {
            actor: player,
            amount: 10.,
        });

        world.run_schedule(Update);

        assert_eq!(world.resource::<TimesMoved>().0, 1);

        world.trigger(MoveX {
            actor: player,
            amount: 5.,
        });

        world.run_schedule(Update);

        assert_eq!(world.resource::<TimesMoved>().0, 2);

        world.trigger(MoveX {
            actor: player,
            amount: -2.,
        });

        world.run_schedule(Update);

        assert_eq!(world.resource::<TimesMoved>().0, 3);

        // Remove the reactivity.

        world
            .entity_mut(player)
            .remove::<NotifyChanged<Transform>>();

        world.trigger(MoveX {
            actor: player,
            amount: -100.,
        });

        assert_eq!(world.resource::<TimesMoved>().0, 3);
    }
}
