## Bevy Notify

A reactive(ish) system for the bevy game engine using relationships.

```rust
use bevy_notify::prelude::*;
use bevy::{prelude::*, ui_widgets::observe};

pub struct Health(pub u8);

let player = commands
    .spawn((
        Name::new("Player"),
        Health(100),
        MonitorSelf,
        observe(|changed: On<ComponentChanged<Health>>, health: Query<&Health>| -> Result<(), BevyError> {
            let current_health = health.get(changed.entity)?;

            println!("My current health is {}", current_health);
        })
    ))
    .id();

commands.spawn((
    Name::new("Doctor"),
    Monitoring(player),
    Notify::<Health>::default(),
    observe(
        |changed: On<ComponentChanged<Health>>,
        mut health: Query<&mut Health>|
        -> Result<(), BevyError> {
            let mut health = health.get_mut(changed.changed)?;

            if health.0 <= 20 {
                health.0 += 20;
            }

            Ok(())
         },
     ),
));
```

## Documentation

Docs aren't great atm, will improve in time but for the time being just make an issue whenever you run into something.

## Future Work

Use many to many relationships once we have them so that multiple entities can be watched at once.
