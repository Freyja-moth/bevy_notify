use bevy_ecs::prelude::*;

#[derive(Component)]
#[relationship_target(relationship = Monitoring)]
/// Contains all the monitors that are watching this entity.
pub struct MoniteredBy(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = MoniteredBy)]
/// A moniter is updated each time the components of the entity it's watching are changed.
///
/// To control which components are watched use [`Notify`]
///
/// ```rust
/// # use bevy_notify::prelude::*;
/// # use bevy::{prelude::*, ui_widgets::observe};
///
/// pub struct Health(pub u8);
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
///     Monitering(player),
///     Notify::<Health>::default(),
///     observe(
///         |changed: On<MoniterChanged<Health>>,
///         mut health: Query<&mut Health>,
///         monitering: Query<&Monitering>|
///         -> Result<(), BevyError> {
///             let &Monitering(player) = monitering.get(changed.entity)?;
///
///             let mut health = health.get_mut(player)?;
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
pub struct Monitoring(pub Entity);

#[derive(Component)]
/// Used to detect changes on the same entity.
pub struct MoniteringSelf;
