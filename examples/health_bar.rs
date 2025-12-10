use bevy::{
    color::palettes::css, input::common_conditions::input_just_pressed, prelude::*,
    ui_widgets::observe,
};
use bevy_notify::prelude::*;

#[derive(Component)]
#[require(
    Node {
        width: percent(100),
        ..Default::default()
    },
    BackgroundColor(css::RED.into()),
)]
pub struct HealthBar;

#[derive(Component)]
#[require(
    Node {
        display: Display::Grid,
        width: percent(100),
        height: percent(100),
        padding: percent(2).all(),
        grid_template_rows: vec![
            GridTrack::percent(2.5),
            GridTrack::flex(1.)
        ],
        grid_template_columns: vec![
            GridTrack::percent(25.),
            GridTrack::flex(1.),
        ],
        ..Default::default()
    }
)]
pub struct UiRoot;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health(u8);

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                harm_player.run_if(input_just_pressed(KeyCode::Space)),
                reset_health.run_if(input_just_pressed(KeyCode::Backspace)),
            ),
        )
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let player = commands
        .spawn((
            Name::new("Player"),
            Player,
            Health(100),
            MoniteringSelf,
            NotifyChanged::<Health>::default(),
            observe(|_: On<Mutation<Health>>| {
                println!("Health has been changed");
            }),
        ))
        .id();

    commands.spawn((
        UiRoot,
        children![(
            HealthBar,
            Monitoring(player),
            NotifyChanged::<Health>::default(),
            observe(
                |mutation: On<Mutation<Health>>,
                 mut health_bar: Query<&mut Node>,
                 health: Query<&Health>|
                 -> Result<(), BevyError> {
                    let mut node = health_bar.get_mut(mutation.entity)?;

                    let health = health.get(mutation.mutated)?;

                    node.width = percent(health.0 as f32);
                    Ok(())
                }
            )
        )],
    ));
}

fn harm_player(mut commands: Commands, mut health: Single<&mut Health, With<Player>>) {
    health.0 = health.0.saturating_sub(10);

    if health.0 == 0 {
        commands.write_message(AppExit::Success);
    }
}

fn reset_health(mut health: Single<&mut Health, With<Player>>) {
    health.0 = 100;
}
