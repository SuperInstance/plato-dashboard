# plato-dashboard

A terminal-based dashboard for monitoring Plato rooms — live sensors, alarms, sparklines, and fleet health, all in your terminal.

No GUI. No web browser. Just text. Designed for captains on boats, ops in data centers, or anyone who needs room awareness at a glance.

## What It Looks Like

```
 ╔══ PLATO DASHBOARD ══╗ [3 rooms, 1Hz refresh]
────────────────────────────────────────────────────────────────────────────────

┌ ● Engine Room (tick: 1000ms, age: 150ms)                            ┐
│ 🟡 Temp  87.3 °C                                                    │
│    ▃▄▅▆▇█ ↑ [80.0..87.3]                                           │
│ 🟢 Humidity  62.1 %                                                 │
│    ▃▃▄▄▅▅▅ ↑ [58.0..62.1]                                          │
│ 🟢 Pressure  1013.2 hPa                                             │
│    ▁▂▃▃▃▃▃ → [1013.0..1013.2]                                       │
└──────────────────────────────────────────────────────────────────────┘

┌ Fleet Overview                                                      ┐
│ Room                 Health     Alarms     Last Tick                │
├──────────────────────────────────────────────────────────────────────┤
│ Engine Room          DEGRADED   1          150ms                    │
│ Bridge               CRITICAL   1          300ms                    │
│ Cargo Hold           CRITICAL   0          30000ms                  │
├──────────────────────────────────────────────────────────────────────┤
│ 🔴 CRITICAL | 3 rooms, 6 sensors, 2 alarms                          │
└──────────────────────────────────────────────────────────────────────┘

┌ Alarms (2 active, 2 unacked)                                        ┐
│ 🟡 ! [1] Engine Room > Temp (Temperature elevated)                  │
│ 🔴 ! [2] Bridge > CO2 (CO2 exceeds safe limit)                      │
├──────────────────────────────────────────────────────────────────────┤
│ History (last 0)                                                     │
└──────────────────────────────────────────────────────────────────────┘

 [Tab] Next room  [Shift+Tab] Prev  [j/k] Scroll  [a] Ack  [q] Quit
 Room 1/3
```

## Architecture

The crate is organized into focused modules:

```
src/
├── lib.rs          # Re-exports and crate documentation
├── main.rs         # Binary entry point with demo data
├── dashboard.rs    # Dashboard: top-level terminal UI with render loop
├── room_panel.rs   # RoomPanel: single room display (sensors, alarms, sparklines)
├── fleet_panel.rs  # FleetPanel: fleet overview with health aggregation
├── alarm_panel.rs  # AlarmPanel: alarm log, history, acknowledgment tracking
├── sparkline.rs    # Sparkline: numeric series → Unicode sparkline characters
└── render.rs       # Terminal rendering utilities (ANSI escape codes, box drawing)
```

### Module Details

#### `Sparkline` — ASCII Sparklines

Converts a `Vec<f64>` into Unicode sparkline characters: `▁▂▃▄▅▆▇█`

```rust
use plato_dashboard::Sparkline;

let sp = Sparkline::new(vec![1.0, 3.0, 5.0, 7.0, 9.0]);
println!("{}", sp.render());           // ▁▂▄▆█
println!("{}", sp.trend_arrow());      // ↑
println!("{:.1}", sp.min());           // 1.0
println!("{:.1}", sp.max());           // 9.0
```

Features:
- Min/max annotation
- Trend arrow (↑ rising, ↓ falling, → flat)
- Handles empty series, single values, and constant series gracefully

#### `RoomPanel` — Single Room Display

Shows detailed status for one room:

- Room name with online/offline indicator (●/○)
- Tick rate and last-seen age
- All sensor values with units
- Per-sensor alarm indicators: 🟢 clear, 🟡 warning, 🔴 critical
- Sparkline of recent sensor history
- Trend arrow for each sensor

```rust
use plato_dashboard::{RoomPanel, RoomState, SensorReading, AlarmSeverity};

let state = RoomState {
    name: "Engine Room".into(),
    tick_rate_ms: 1000,
    last_tick_age_ms: 200,
    sensors: vec![
        SensorReading {
            name: "Temp".into(),
            value: 22.5,
            unit: "°C".into(),
            alarm: AlarmSeverity::Clear,
        },
    ],
    history: vec![vec![20.0, 21.0, 22.0, 22.5]],
    online: true,
};

let panel = RoomPanel::new(state);
let lines = panel.render(80);
```

#### `FleetPanel` — Fleet Overview

One line per room with aggregate health:

```
┌ Fleet Overview                                      ┐
│ Room         Health     Alarms    Last Tick         │
│ Engine Room  HEALTHY   0         150ms              │
│ Bridge       DEGRADED  1         300ms              │
│ Cargo Hold   OFFLINE   0         30000ms            │
│ 🟡 DEGRADED | 3 rooms, 5 sensors, 1 alarm          │
└─────────────────────────────────────────────────────┘
```

Health logic:
- **Green (HEALTHY)** — All sensors clear, room online
- **Yellow (DEGRADED)** — At least one warning alarm
- **Red (CRITICAL)** — Any critical alarm or room offline

Aggregate health is the worst of any individual room.

#### `AlarmPanel` — Alarm Log & Acknowledgments

Tracks active alarms and resolved alarm history:

```rust
use plato_dashboard::{AlarmPanel, AlarmEvent, AlarmSeverity};

let mut panel = AlarmPanel::new(50); // max 50 history entries

panel.add_alarm(AlarmEvent {
    id: 1,
    room_name: "Engine Room".into(),
    sensor_name: "Temp".into(),
    severity: AlarmSeverity::Warning,
    message: "Temperature elevated".into(),
    timestamp_secs: 1000,
    acknowledged: false,
});

panel.acknowledge(1);   // Mark as acknowledged
panel.resolve(1);       // Move to history
```

Features:
- Active alarm display with severity indicators
- Acknowledgment tracking (✓ acknowledged, ! unacknowledged)
- Automatic history trimming
- Resolve moves alarms to history with timestamp

#### `Render` — Terminal Utilities

Pure ANSI escape code rendering — no external TUI library dependency:

- Clear screen, cursor positioning
- Foreground/background color support (Red, Green, Yellow, Blue, Magenta, Cyan, White)
- Box-drawing characters (┌─┐│└┘┬┴├┤┼)
- Text alignment (center, pad/truncate)
- Cursor show/hide

#### `Dashboard` — The Full UI

Ties everything together:

```rust
use plato_dashboard::{Dashboard, FleetConfig, RoomState, SensorReading, AlarmSeverity};

let config = FleetConfig::new(vec![
    // ... room states
]);

let mut dashboard = Dashboard::new(config);
dashboard.set_running(true);
println!("{}", dashboard.render_full());
```

Keyboard controls:
| Key | Action |
|-----|--------|
| `Tab` | Switch to next room |
| `Shift+Tab` | Switch to previous room |
| `j` | Scroll history down |
| `k` | Scroll history up |
| `a` | Acknowledge current alarm |
| `q` | Quit |

## Connection to plato-fleet-manager

This dashboard is designed to work with `plato-fleet-manager`, which provides the fleet manifest and real-time room data. The typical flow:

1. **`plato-fleet-manager`** discovers and manages Plato rooms on the network
2. It exposes room states, sensor readings, and alarm events
3. **`plato-dashboard`** connects to the fleet manager and renders live data

The `FleetConfig` struct accepts room states directly, making it easy to feed data from any source — whether that's the fleet manager, a mock data source, or a file.

### Fleet Manifest Format

The dashboard expects a `FleetConfig` containing `Vec<RoomState>`, where each `RoomState` includes:

```rust
pub struct RoomState {
    pub name: String,           // Human-readable room name
    pub tick_rate_ms: u64,      // How often the room sends updates
    pub last_tick_age_ms: u64,  // Age of the most recent update
    pub sensors: Vec<SensorReading>,
    pub history: Vec<Vec<f64>>, // Per-sensor history for sparklines
    pub online: bool,           // Whether the room is reachable
}
```

## Pure Standard Library

This crate uses **zero TUI dependencies**. No crossterm, no termion, no tui-rs. All rendering is done with:

- ANSI escape codes for colors and cursor control
- Unicode box-drawing characters for panel borders
- Unicode block characters for sparklines

This means it works in any terminal that supports basic ANSI — which is all of them.

## Building

```bash
cargo build
```

## Running the Demo

```bash
cargo run
```

This renders a static snapshot with 3 mock rooms. In production, you'd wire this into a render loop connected to live room data.

## Running Tests

```bash
cargo test
```

50 tests covering all modules: sparkline rendering, room panels, fleet health aggregation, alarm lifecycle, box drawing, color formatting, and full dashboard integration.

## License

MIT
