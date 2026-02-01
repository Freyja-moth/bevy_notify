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
/// Indicates that the component [`C`] has been added to an entity watched by a monitor.
///
/// See [`NotifyAdded<C>`] for more information on how this is triggered.
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
/// Adding this component to a entity will cause it to react to component [`C`] being added to
/// an entity with [`Addition<C>`]
///
/// By default this will react to changes on **all** entities. See [`Monitor`], and [`MonitorSelf`]
/// for restricting this.
///
/// # Technical info
///
/// Adding this component to an entity will spawn an [`Observer`] for event [`On<Add, C>`], this is
/// only done once.
///
/// When all instances of this component in the world are removed the observer will be despawned.
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
    local_monitors: Query<Entity, (With<NotifyAdded<C>>, With<MonitorSelf>)>,
    monitors: Query<(Entity, &Monitor), With<NotifyAdded<C>>>,
    global_monitors: Query<Entity, (With<NotifyAdded<C>>, Without<Monitor>, Without<MonitorSelf>)>,
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
        .filter(|(_, Monitor(entity))| *entity == add.entity)
        .for_each(|(entity, &Monitor(added))| {
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct Player;

    #[derive(Component)]
    pub struct Poisoned;

    #[test]
    fn check_for_addition() {
        #[derive(Resource, Default, Debug)]
        pub struct TimesPoisoned(usize);

        let mut world = World::new();

        world.init_resource::<TimesPoisoned>();

        let player = world
            .spawn((Player, MonitorSelf, NotifyAdded::<Poisoned>::default()))
            .observe(
                |_: On<Addition<Poisoned>>, mut status_effect_count: ResMut<TimesPoisoned>| {
                    status_effect_count.0 += 1;
                },
            )
            .id();

        assert_eq!(world.resource::<TimesPoisoned>().0, 0);

        world.entity_mut(player).insert(Poisoned);

        assert_eq!(world.resource::<TimesPoisoned>().0, 1);

        world.entity_mut(player).remove::<Poisoned>();

        assert_eq!(world.resource::<TimesPoisoned>().0, 1);

        world.entity_mut(player).insert(Poisoned);

        assert_eq!(world.resource::<TimesPoisoned>().0, 2);

        // Remove the reactivity.

        world
            .entity_mut(player)
            .remove::<NotifyAdded<Poisoned>>()
            .insert(Poisoned);

        assert_eq!(world.resource::<TimesPoisoned>().0, 2);
    }
}
