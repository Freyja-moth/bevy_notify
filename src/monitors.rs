use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Component, Reflect, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
#[relationship_target(relationship = Monitor)]
/// Contains all the monitors that are watching this entity.
pub struct MonitoredBy(Vec<Entity>);

#[derive(Component, Reflect, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[relationship(relationship_target = MonitoredBy)]
/// A moniter is updated each time the components of the entity it's watching are changed.
///
/// To control which components are watched use [`Notify`]
///
/// ```rust
/// # use bevy_notify::prelude::*;
/// # use bevy::{prelude::*, ui_widgets::observe};
///
/// # #[derive(Component)]
/// # pub struct Health(pub u8);
///
/// # fn showcase(mut commands: Commands) {
/// let player = commands
///     .spawn((
///         Name::new("Player"),
///         Health(100)
///     ))
///     .id();
///
/// commands.spawn((
///     Name::new("Doctor"),
///     Monitor(player),
///     NotifyChanged::<Health>::default(),
///     observe(
///         |mutation: On<Mutation<Health>>,
///         mut health: Query<&mut Health>|
///         -> Result<(), BevyError> {
///             let mut health = health.get_mut(mutation.mutated)?;
///
///             if health.0 <= 20 {
///                 health.0 += 20;
///             }
///
///             Ok(())
///         },
///     ),
/// ));
/// # }
/// ```
pub struct Monitor(pub Entity);

#[derive(Component, Reflect, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
/// Used to detect changes on the same entity.
pub struct MonitorSelf;

#[cfg(test)]
mod test {
    /// TODO: Test all types of reactivity.
    use crate::prelude::*;
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct Tester;

    #[derive(Resource, Default)]
    pub struct TesterAdded(usize);

    #[test]
    fn test_local_monitor() {
        let mut world = World::new();

        world.init_resource::<TesterAdded>();

        let monitor = world
            .spawn((MonitorSelf, NotifyAdded::<Tester>::default()))
            .observe(
                |_: On<Addition<Tester>>, mut tester_added: ResMut<TesterAdded>| {
                    tester_added.0 += 1;
                },
            )
            .id();

        let empty = world.spawn_empty().id();

        world.entity_mut(empty).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 0);

        world.entity_mut(monitor).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 1);
    }

    #[test]
    fn test_related_monitor() {
        let mut world = World::new();

        world.init_resource::<TesterAdded>();

        let empty = world.spawn_empty().id();

        let monitor = world
            .spawn((Monitor(empty), NotifyAdded::<Tester>::default()))
            .observe(
                |_: On<Addition<Tester>>, mut tester_added: ResMut<TesterAdded>| {
                    tester_added.0 += 1;
                },
            )
            .id();

        world.entity_mut(monitor).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 0);

        world.entity_mut(empty).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 1);
    }

    #[test]
    fn test_mixed_monitor() {
        let mut world = World::new();

        world.init_resource::<TesterAdded>();

        let unrelated = world.spawn_empty().id();

        let empty = world.spawn_empty().id();

        let monitor = world
            .spawn((
                Monitor(empty),
                MonitorSelf,
                NotifyAdded::<Tester>::default(),
            ))
            .observe(
                |_: On<Addition<Tester>>, mut tester_added: ResMut<TesterAdded>| {
                    tester_added.0 += 1;
                },
            )
            .id();

        world.entity_mut(monitor).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 1);

        world.entity_mut(empty).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 2);

        world.entity_mut(unrelated).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 2);
    }

    #[test]
    fn test_global_monitor() {
        let mut world = World::new();

        world.init_resource::<TesterAdded>();

        let empty_two = world.spawn_empty().id();

        let empty_one = world.spawn_empty().id();

        let monitor = world
            .spawn((NotifyAdded::<Tester>::default(),))
            .observe(
                |_: On<Addition<Tester>>, mut tester_added: ResMut<TesterAdded>| {
                    tester_added.0 += 1;
                },
            )
            .id();

        world.entity_mut(monitor).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 1);

        world.entity_mut(empty_one).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 2);

        world.entity_mut(empty_two).insert(Tester);

        assert_eq!(world.resource::<TesterAdded>().0, 3);
    }
}
