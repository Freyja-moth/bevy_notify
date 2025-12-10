## Bevy Notify

A reactive(ish) system for the bevy game engine using relationships.

```rust
use bevy_notify::prelude::*;
use bevy::{prelude::*, ui_widgets::observe};

pub struct Health(pub u8);

let player = commands
    .spawn((
        Name::new("Player"),
        Health(100)
    ))
    .id();

commands.spawn((
    Name::new("Doctor"),
    Monitering(player),
    Notify::<Health>::default(),
    observe(
        |changed: On<MoniterChanged<Health>>,
        mut health: Query<&mut Health>,
        monitering: Query<&Monitering>|
        -> Result<(), BevyError> {
            let &Monitering(player) = monitering.get(changed.entity)?;

            let mut health = health.get_mut(player)?;

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


