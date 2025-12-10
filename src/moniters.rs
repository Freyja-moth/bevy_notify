use bevy_app::Update;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use std::marker::PhantomData;

#[derive(Resource)]
/// Used to indicate that the component [`C`] is being watched by a system to prevent systems from
/// being added multiple times.
struct Watching<C>(PhantomData<C>);

#[derive(EntityEvent)]
/// Indicates that the component [`C`] on the monitered entity has changed.
pub struct MoniterChanged<C: Component> {
    pub entity: Entity,
    pub(crate) _phantom: PhantomData<C>,
}

#[derive(Component)]
#[relationship_target(relationship = Monitering)]
/// Contains all the moniters that are watching this entity.
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
pub struct Monitering(pub Entity);

#[derive(Component)]
#[component(on_add = register_component_moniter::<C>)]
/// Specifies that a moniter should react to all changed to [`C`] on the monitered entity.
pub struct Notify<C: Component>(PhantomData<C>);
impl<C: Component> Default for Notify<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

fn watch_for_change<C: Component>(
    mut commands: Commands,
    monitered: Populated<&MoniteredBy, Changed<C>>,
    moniters: Populated<Entity, (With<Notify<C>>, With<Monitering>)>,
) {
    monitered
        .iter()
        .flat_map(|monitered_by| monitered_by.collection())
        // Ensure that only moniters with [`Notify<C>`] are triggered
        .filter(|&&entity| moniters.contains(entity))
        .for_each(|&entity| {
            commands.trigger(MoniterChanged {
                entity,
                _phantom: PhantomData::<C>,
            })
        });
}

fn register_component_moniter<C: Component>(mut world: DeferredWorld, _: HookContext) {
    if world.contains_resource::<Watching<C>>() {
        return;
    }

    world.commands().queue(|world: &mut World| {
        world.schedule_scope(Update, |_, schedule| {
            schedule.add_systems(watch_for_change::<C>);
        });
        world.insert_resource(Watching::<C>(PhantomData));
    });
}
