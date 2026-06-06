# Plug & Play — Plato Dashboard

Copy-paste templates for the three most common patterns.

## Pattern 1: Static Dashboard Snapshot

```rust
use plato_dashboard::*;

fn main() {
    let rooms = vec![
        RoomState {
            name: "Engine".into(),
            tick_rate_ms: 1000,
            last_tick_age_ms: 200,
            sensors: vec![
                SensorReading {
                    name: "Temp".into(), value: 72.5, unit: "°C".into(),
                    alarm: AlarmSeverity::Clear,
                },
            ],
            history: vec![vec![70.0, 71.0, 72.0, 72.5]],
            online: true,
        },
    ];

    let config = FleetConfig::new(rooms);
    let dashboard = Dashboard::new(config);
    println!("{}", dashboard.render_full());
}
```

## Pattern 2: Dashboard with Alarms

```rust
use plato_dashboard::*;

fn main() {
    let config = FleetConfig::new(vec![
        RoomState {
            name: "Engine".into(), tick_rate_ms: 1000, last_tick_age_ms: 200,
            sensors: vec![SensorReading {
                name: "Temp".into(), value: 95.0, unit: "°C".into(),
                alarm: AlarmSeverity::Critical,
            }],
            history: vec![vec![80.0, 85.0, 90.0, 95.0]],
            online: true,
        },
    ]);

    let mut dashboard = Dashboard::new(config);

    // Add alarm
    dashboard.alarm_panel_mut().add_alarm(AlarmEvent {
        id: 1, room_name: "Engine".into(), sensor_name: "Temp".into(),
        severity: AlarmSeverity::Critical, message: "Overheat!".into(),
        timestamp_secs: 1000, acknowledged: false,
    });

    // Acknowledge and resolve
    dashboard.alarm_panel_mut().acknowledge(1);
    dashboard.alarm_panel_mut().resolve(1);

    println!("{}", dashboard.render_full());
}
```

## Pattern 3: Sparkline + Fleet Health

```rust
use plato_dashboard::*;

// Sparkline from data
let spark = Sparkline::new(vec![20.0, 25.0, 30.0, 35.0, 40.0]);
println!("Trend: {} {}", spark.render(), spark.trend_arrow());
// ▁▂▄▆█ ↑

// Fleet health check
let rooms = vec![
    RoomState {
        name: "A".into(), tick_rate_ms: 1000, last_tick_age_ms: 100,
        sensors: vec![SensorReading {
            name: "S".into(), value: 10.0, unit: "°C".into(),
            alarm: AlarmSeverity::Clear,
        }],
        history: vec![], online: true,
    },
];
let fleet = FleetPanel::new(rooms);
let stats = fleet.stats();
println!("Fleet: {} rooms, {} sensors, health: {}",
         stats.total_rooms, stats.total_sensors, stats.health.label());
// Fleet: 1 rooms, 1 sensors, health: HEALTHY
```

## Quick Reference

| What | Code |
|------|------|
| Create room state | `RoomState { name, tick_rate_ms, last_tick_age_ms, sensors, history, online }` |
| Sensor reading | `SensorReading { name, value, unit, alarm }` |
| Alarm severity | `AlarmSeverity::Clear / Warning / Critical` |
| Sparkline | `Sparkline::new(vec![...]).render()` |
| Trend arrow | `spark.trend_arrow()` → `↑ ↓ →` |
| Fleet health | `FleetPanel::room_health(&room)` → `FleetHealth::Green/Yellow/Red` |
| Fleet stats | `fleet.stats()` → `FleetStats { total_rooms, total_sensors, total_alarms, health }` |
| Add alarm | `dashboard.alarm_panel_mut().add_alarm(event)` |
| Acknowledge | `dashboard.alarm_panel_mut().acknowledge(id)` |
| Resolve | `dashboard.alarm_panel_mut().resolve(id)` |
| Render dashboard | `dashboard.render_full()` → ANSI `String` |
| Switch rooms | `dashboard.next_room()` / `dashboard.prev_room()` |
| Box drawing | `use plato_dashboard::render::box_drawing as bd` |
| Colors | `Render::colorize(text, Color::Red)` |
