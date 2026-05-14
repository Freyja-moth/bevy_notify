use bevy::{
    color::palettes::css,
    feathers::{
        FeathersPlugins,
        containers::{pane, pane_body, pane_header},
        controls::FeathersButton,
        dark_theme::create_dark_theme,
        display::label,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    prelude::*,
    ui_widgets::Activate,
};
use bevy_monitors::prelude::*;

#[derive(EntityEvent)]
pub struct Hurt {
    entity: Entity,
    damage: u8,
}

#[derive(EntityEvent)]
pub struct Died(Entity);

#[derive(SceneComponent, Clone, Default)]
pub struct HealthBar;

impl HealthBar {
    pub fn scene() -> impl Scene {
        bsn! {
            HealthBar
            Node {
                height: px(20),
                max_width: percent(30)
            }
            BackgroundColor(css::RED)
            NotifyChanged::<Health>::default()
            on(update_bar)
        }
    }
}

fn update_bar(
    mutation: On<Mutation<Health>>,
    mut health_bar: Query<&mut Node>,
    health: Query<&Health>,
) -> Result<(), BevyError> {
    let mut node = health_bar.get_mut(mutation.entity)?;

    let health = health.get(mutation.mutated)?;

    let max_width = if let Val::Percent(percent) = node.max_width {
        percent
    } else {
        100.
    };

    node.width = percent(health.0 as f32 * (max_width / 100.));
    Ok(())
}

#[derive(Component, Clone, Default)]
pub struct Player;

#[derive(Component, Clone, Default)]
pub struct Health(u8);

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let player = commands.spawn_scene(player()).id();

    commands.spawn_scene(ui(player));
}

fn ui(player: Entity) -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
            padding: px(8),
            row_gap: px(16),
            width: percent(100),
            height: percent(100)
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            :actions_pane(),
            :HealthBar
            Monitor(player)
        ]
    }
}

fn actions_pane() -> impl Scene {
    bsn! {
        :pane Children [
            :pane_header Children [
                :label("Actions")
            ],
            :pane_body Children [
                :FeathersButton {
                    @caption: {bsn! {
                        Text("Attack") ThemedText
                    }}
                }
                on(|_: On<Activate>, mut commands: Commands, player: Single<Entity, With<Player>>| {
                    commands.trigger(Hurt {
                        entity: *player,
                        damage: 10,
                    });
                }),
                :FeathersButton {
                    @caption: {bsn! {
                        Text("Heal") ThemedText
                    }}
                }
                on(|_: On<Activate>, mut health: Single<&mut Health, With<Player>>| {
                    health.0 = 100;
                })
            ]
        ]
    }
}

fn player() -> impl Scene {
    bsn! {
        #Player
        Player
        Health(100)
        MonitorSelf
        NotifyChanged::<Health>::default()
        on(|_: On<Died>, mut commands: Commands| {
            commands.write_message(AppExit::Success);
        })
        on(|player: On<Mutation<Health>>, mut commands: Commands, health: Query<&Health>| -> Result<(), BevyError> {
            let Health(health) = health.get(player.entity)?;

            if *health == 0 {
                commands.entity(player.entity).trigger(Died);
            }

            Ok(())
        })
        on(|player: On<Hurt>, mut health: Query<&mut Health>| -> Result<(), BevyError> {
            let mut health = health.get_mut(player.entity)?;

            health.0 = health.0.saturating_sub(player.damage);

            Ok(())
        })
    }
}
