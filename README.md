# PLATO Dashboard

**Fleet dashboard rendering for the PLATO nervous system** — a terminal-based observability platform that aggregates room states, autonomy metrics, sensor readings, alarm panels, and fleet-wide health into structured ANSI-rendered panels with sparklines, color coding, and box-drawing borders.

## Why It Matters

The PLATO Dashboard is the **control room** for the SuperInstance fleet nervous system:

- **Real-time fleet monitoring** — track dozens of autonomous rooms (ship compartments, server racks, robotic cells) simultaneously.
- **Alarm management** — active alarm tracking with acknowledgment workflow and resolution history.
- **Sparkline visualization** — inline trend charts (▁▂▃▄▅▆▇█) for sensor data without external charting tools.
- **Health aggregation** — fleet-level health rollup (Green / Yellow / Red) from per-room sensor status.
- **Multi-format output** — structured data for text, JSON, or HTML rendering.

## How It Works

### Panel Architecture

The dashboard composes independent panels:

```
┌─ RoomPanel ────────────────────────┐
│ ● Engine (tick: 1000ms, age: 200ms) │
│ 🟢 Temp   22.5 °C                   │
│    ▁▂▃▄▅▆ ↑ [18.0..22.5]           │
│ 🟡 Pressure  3.2 bar                 │
└─────────────────────────────────────┘
┌─ FleetPanel ───────────────────────┐
│ Room                 Health  Alarms │
│ Engine               DEGRADED  1    │
│ Bridge               HEALTHY   0    │
│ Hold                 CRITICAL  2    │
├─ 🟡 DEGRADED | 3 rooms, 8 sensors  ─┤
└─────────────────────────────────────┘
┌─ AlarmPanel ───────────────────────┐
│ ⚠️ ! [1] Engine > Temp (Overheat)   │
│ 🔴 ! [2] Hold > Pressure (Critical) │
│─ History ───────────────────────────│
│    [3] Galley > Humidity ✓ @ t=990  │
└─────────────────────────────────────┘
```

### Sparkline Rendering

Numeric series are rendered to Unicode block characters:

```
Given values [v₁, v₂, …, vₙ]:
  min = min(values), max = max(values)
  For each vᵢ:
    normalized = (vᵢ - min) / (max - min)
    idx = round(normalized × 7)  → [0, 7]
    char = SPARK_CHARS[idx]      → ▁▂▃▄▅▆▇█
```

Edge cases:
- Empty series → `""`
- Single value → `▄` (middle character)
- All same values → `▄` repeated (no variation)

### Fleet Health Computation

```
For each room:
  if !room.online          → Red
  if any sensor == Critical → Red
  if any sensor == Warning  → Yellow
  else                      → Green

Fleet health = max-merge of room healths:
  any Red → Red
  any Yellow (no Red) → Yellow
  all Green → Green
```

### Alarm Lifecycle

```
1. AlarmEvent created (active, unacknowledged)
2. Operator acknowledges → acknowledged = true
3. Condition resolves → moved to history
4. History trimmed to max_history entries
```

### Complexity

| Operation | Time |
|-----------|------|
| Dashboard render | O(R · S) where R = rooms, S = sensors per room |
| Fleet health rollup | O(R) |
| Alarm resolve | O(A) where A = active alarms |
| Sparkline render | O(n) where n = series length |

## Quick Start

```rust
use plato_dashboard::*;

fn main() {
    let rooms = vec![
        RoomSummary::new("Engine", RoomType::Engine, RoomStatus::Warning, 89.1, 0.72, 5),
        RoomSummary::new("Bridge", RoomType::Bridge, RoomStatus::Ok, 98.4, 0.95, 2),
        RoomSummary::new("Hold", RoomType::Hold, RoomStatus::Critical, 45.3, 0.34, 12),
    ];

    let alerts = vec![
        AlertSummary::new("Engine", "overheat", "Temp above threshold", Severity::Warning, 1000),
        AlertSummary::new("Hold", "pressure", "Pressure critical", Severity::Critical, 1005),
    ];

    // Dashboard renders to ANSI-colored string
    let output = render_dashboard(&rooms, &alerts);
    println!("{}", output);
}
```

## API

### Core Types

| Type | Description |
|------|-------------|
| `Severity` | Info < Warning < Critical < Emergency |
| `AlertSummary` | Room-associated alert with ack tracking |
| `RoomStatus` | Ok / Warning / Critical / Offline |
| `RoomSummary` | Per-room state with autonomy % and health score |
| `FleetSummary` | Aggregate fleet statistics |

### Panels

| Module | Description |
|--------|-------------|
| `room_panel` | Single-room detail with sensors, alarms, sparklines |
| `fleet_panel` | Fleet overview table with health rollup |
| `alarm_panel` | Active alarms + resolution history |
| `sparkline` | Unicode block-character trend charts |
| `render` | ANSI colors, box-drawing, terminal control |
| `dashboard` | Top-level composition + keyboard navigation |

### Render Utilities

```rust
use plato_dashboard::render::{Render, Color, box_drawing};

Render::colorize("text", Color::Red);           // ANSI red
Render::pad("text", 20);                         // pad/truncate to width
Render::draw_box("Title", 30, 5);               // box-drawing border
Render::clear_screen();                          // \x1b[2J\x1b[H
```

## Architecture Notes

The rendering system uses pure ANSI escape codes — no external TUI framework (crossterm, termion, ncurses). This makes the output portable across any terminal emulator and embeddable in log output.

The box-drawing characters (┌┐└┘─│┬┴├┤┼) use Unicode box-drawing range U+2500–U+257F.

The **γ + η = C** ternary classification: each fleet room is **(γ) operational** (status Ok/Warning — functional with possible degradation) or **(η) non-operational** (status Critical/Offline — requires intervention). The fleet health rollup uses max-merge: any η poisons the entire fleet to Red.

## References

1. Nielsen, J. (1993). "Response times: The 3 important limits." *Usability Engineering*. — 0.1s/1s/10s interaction thresholds for dashboards.
2. Tufte, E. R. (2001). *The Visual Display of Quantitative Information* (2nd ed.). Graphics Press. — Sparkline philosophy.
3. Few, S. (2006). *Information Dashboard Design*. O'Reilly. — Dashboard layout principles.
4. Williams, R. (2014). "ANSI escape sequences." *ECMA-48* standard.
5. Kim, G. et al. (2016). *The Phoenix Project*. IT Revolution Press. — Fleet observability philosophy.

## License

MIT
