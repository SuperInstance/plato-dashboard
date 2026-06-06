# Tutorial — Building a Custom Fleet Dashboard

This tutorial walks you through creating a terminal dashboard that displays live sensor data from Plato rooms. You'll start from scratch and end with a working fleet monitor.

**Prerequisites:** Rust toolchain, this crate in your project.

## Step 1: Create the Project

```bash
cargo new my_dashboard && cd my_dashboard
```

Add to `Cargo.toml`:

```toml
[dependencies]
plato-dashboard = { path = "/path/to/plato-dashboard" }
```

## Step 2: Create Room States

In `src/main.rs`, define the rooms you want to monitor:

```rust
use plato_dashboard::*;

fn main() {
    // Define room states
    let rooms = vec![
        RoomState {
            name: "Engine Room".into(),
            tick_rate_ms: 1000,
            last_tick_age_ms: 150,
            sensors: vec![
                SensorReading {
                    name: "Temp".into(),
                    value: 87.3,
                    unit: "°C".into(),
                    alarm: AlarmSeverity::Warning,
                },
                SensorReading {
                    name: "Humidity".into(),
                    value: 62.1,
                    unit: "%".into(),
                    alarm: AlarmSeverity::Clear,
                },
                SensorReading {
                    name: "Pressure".into(),
                    value: 1013.2,
                    unit: "hPa".into(),
                    alarm: AlarmSeverity::Clear,
                },
            ],
            history: vec![
                vec![80.0, 82.0, 84.0, 85.0, 86.0, 87.3],  // temp history
                vec![58.0, 59.0, 60.0, 61.0, 61.5, 62.1],  // humidity history
                vec![1013.0, 1013.0, 1013.1, 1013.1, 1013.2, 1013.2],  // pressure history
            ],
            online: true,
        },
        RoomState {
            name: "Bridge".into(),
            tick_rate_ms: 500,
            last_tick_age_ms: 300,
            sensors: vec![
                SensorReading {
                    name: "CO2".into(),
                    value: 1200.0,
                    unit: "ppm".into(),
                    alarm: AlarmSeverity::Critical,
                },
            ],
            history: vec![vec![800.0, 900.0, 1000.0, 1100.0, 1150.0, 1200.0]],
            online: true,
        },
        RoomState {
            name: "Cargo Hold".into(),
            tick_rate_ms: 2000,
            last_tick_age_ms: 30000,  // 30 seconds — might be offline
            sensors: vec![
                SensorReading {
                    name: "Temp".into(),
                    value: 4.2,
                    unit: "°C".into(),
                    alarm: AlarmSeverity::Clear,
                },
            ],
            history: vec![vec![4.0, 4.1, 4.1, 4.2, 4.2, 4.2]],
            online: false,
        },
    ];

    println!("Loaded {} rooms", rooms.len());
}
```

## Step 3: Render Individual Panels

Before connecting to the dashboard, let's render each panel independently:

```rust
// Sparkline demo
let temp_history = vec![72.0, 74.0, 76.0, 78.0, 80.0, 83.0, 86.0, 87.3];
let spark = Sparkline::new(temp_history.clone());
println!("Temperature: {} {}", spark.render(), spark.trend_arrow());
println!("Range: {:.1} – {:.1}", spark.min(), spark.max());
```

Output:
```
Temperature: ▃▃▄▅▆▇██ ↑
Range: 72.0 – 87.3
```

```rust
// Fleet panel
let fleet = FleetPanel::new(rooms.clone());
for line in fleet.render(80) {
    println!("{}", line);
}
```

## Step 4: Build the Full Dashboard

Now wire everything together:

```rust
// Create fleet config
let config = FleetConfig::new(rooms);

// Create dashboard
let mut dashboard = Dashboard::new(config);

// Add some alarms
dashboard.alarm_panel_mut().add_alarm(AlarmEvent {
    id: 1,
    room_name: "Engine Room".into(),
    sensor_name: "Temp".into(),
    severity: AlarmSeverity::Warning,
    message: "Temperature elevated".into(),
    timestamp_secs: 1000,
    acknowledged: false,
});

dashboard.alarm_panel_mut().add_alarm(AlarmEvent {
    id: 2,
    room_name: "Bridge".into(),
    sensor_name: "CO2".into(),
    severity: AlarmSeverity::Critical,
    message: "CO2 exceeds safe limit".into(),
    timestamp_secs: 1010,
    acknowledged: false,
});

// Render
println!("{}", dashboard.render_full());
```

## Step 5: Add Interactive Controls

For a real application, wrap the dashboard in a render loop:

```rust
use std::io::{self, Read};
use std::thread;
use std::time::Duration;

fn main() {
    // ... create dashboard as above ...
    dashboard.set_running(true);

    // Simple render loop
    let mut counter = 0u64;
    while dashboard.is_running() {
        // Simulate updating sensor values
        counter += 1;
        // In production: dashboard.rooms[i].sensors[j].value = live_value;

        // Render
        println!("{}", dashboard.render_full());

        // Simulate keyboard input
        // In production: use crossterm or termion for real keyboard handling
        if counter > 5 {
            // Acknowledge an alarm
            dashboard.alarm_panel_mut().acknowledge(1);
        }

        if counter > 10 {
            // Resolve an alarm
            dashboard.alarm_panel_mut().resolve(2);
        }

        if counter > 15 {
            dashboard.set_running(false);
        }

        thread::sleep(Duration::from_millis(1000));
    }
}
```

## Step 6: Build a Room Panel from Scratch

Create a custom panel for specialized data:

```rust
use plato_dashboard::render::{Render, box_drawing as bd, Color};

fn render_custom_panel(title: &str, data: &[(String, String)], width: usize) -> Vec<String> {
    let inner = width.saturating_sub(2);
    let mut lines = Vec::new();

    // Title bar
    lines.push(format!(
        "{}{}{}",
        bd::TOP_LEFT,
        Render::pad(&format!(" {} ", title), inner),
        bd::TOP_RIGHT
    ));

    // Data rows
    for (key, value) in data {
        let row = format!(" {:<20} {}", key, value);
        lines.push(format!(
            "{}{}{}",
            bd::VERTICAL,
            Render::pad(&row, inner),
            bd::VERTICAL
        ));
    }

    // Bottom
    lines.push(format!(
        "{}{}{}",
        bd::BOTTOM_LEFT,
        Render::horizontal_line(inner),
        bd::BOTTOM_RIGHT
    ));

    lines
}

// Use it
let data = vec![
    ("Uptime".into(), "14d 3h 22m".into()),
    ("Packets".into(), "1,234,567".into()),
    ("Errors".into(), "3".into()),
];
for line in render_custom_panel("Network Stats", &data, 50) {
    println!("{}", line);
}
```

Output:
```
┌ Network Stats ──────────────────────────────────┐
│ Uptime               14d 3h 22m                 │
│ Packets              1,234,567                   │
│ Errors               3                           │
└──────────────────────────────────────────────────┘
```

## What You Built

- ✅ Multi-room fleet dashboard with live sensor display
- ✅ Sparkline visualization with trend arrows
- ✅ Fleet health aggregation (green/yellow/red)
- ✅ Alarm lifecycle management (active → acknowledged → resolved)
- ✅ Custom panel rendering with box-drawing characters
- ✅ Zero external TUI dependencies

## Next Steps

- Wire into `plato-fleet-manager` for live data
- Add keyboard handling with crossterm for interactive use
- Create custom panels for domain-specific visualizations
- Run `cargo run` in this crate to see the full demo with 3 rooms
