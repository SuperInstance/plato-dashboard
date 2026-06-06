# Developer Guide — Plato Dashboard

## Architecture Overview

The Plato Dashboard is a zero-dependency terminal UI for monitoring Plato rooms. No crossterm, no termion, no tui-rs — just ANSI escape codes, Unicode box-drawing characters, and Unicode block elements for sparklines. It renders to a `String`, making it trivially testable.

### Module Map

```
src/
├── lib.rs          — Re-exports and crate-level docs
├── main.rs         — Binary entry point with demo data
├── dashboard.rs    — Dashboard: top-level UI, room switching, render orchestration
├── room_panel.rs   — RoomPanel: single room with sensors, alarms, sparklines
├── fleet_panel.rs  — FleetPanel: one-line-per-room overview with health aggregation
├── alarm_panel.rs  — AlarmPanel: active alarms, history, acknowledgment tracking
├── sparkline.rs    — Sparkline: Vec<f64> → Unicode sparkline characters
└── render.rs       — Terminal utilities: ANSI codes, colors, box drawing, padding
```

### Data Flow

```
FleetConfig (Vec<RoomState>)
      ↓
Dashboard
├── RoomPanel (selected room: sensors, sparklines, alarms)
├── FleetPanel (all rooms: health, alarm counts, tick age)
└── AlarmPanel (active + history with ack tracking)
      ↓
render_full() → String (ANSI-encoded terminal output)
```

### Module Walkthrough

#### `render.rs` — Terminal Primitives

Pure functions for ANSI output. No state, no side effects.

- **`Render::clear_screen()`** — ANSI escape to clear terminal.
- **`Render::colorize(text, Color)`** — Wrap text in ANSI color codes. Colors: Red, Green, Yellow, Blue, Magenta, Cyan, White.
- **`Render::pad(text, width)`** — Pad or truncate text to exact width.
- **`Render::horizontal_line(width)`** — Draw a `────` line.
- **`box_drawing`** module — Constants for `┌─┐│└┘┬┴├┤┼`.

All rendering returns `String`. No direct terminal I/O — the caller decides whether to print.

#### `sparkline.rs` — Numeric Visualization

Converts `Vec<f64>` into `▁▂▃▄▅▆▇█`:

- **Normalization**: Map values to 0–7 index range using min/max.
- **Edge cases**: Empty → `""`, single value → middle char `▅`, constant → `▄▄▄▄`.
- **Trend**: `trend_arrow()` compares first/last with 1% threshold: `↑` rising, `↓` falling, `→` flat.
- **Annotations**: `render_with_annotations()` appends min/max and trend.

#### `room_panel.rs` — Single Room Display

`RoomPanel` renders one room with:

- **Header**: Room name, online status (`●`/`○`), tick rate, last-seen age.
- **Sensors**: Value, unit, alarm indicator (🟢/🟡/🔴), sparkline, trend arrow.
- **Alarm severity**: `Clear` (green), `Warning` (yellow), `Critical` (red).

Input: `RoomState` struct containing sensor readings and per-sensor history arrays.

#### `fleet_panel.rs` — Fleet Overview

`FleetPanel` shows one line per room with aggregate health:

**Health logic** (per room):
1. Offline → Red
2. Any critical alarm → Red
3. Any warning alarm → Yellow
4. Otherwise → Green

**Aggregate health**: worst of all rooms. One red room makes the fleet red.

`FleetStats` tracks total rooms, sensors, and active alarms.

#### `alarm_panel.rs` — Alarm Lifecycle

`AlarmPanel` manages alarm state through three phases:

1. **Active**: New alarms via `add_alarm()`. Tracked with acknowledgment status.
2. **Acknowledged**: `acknowledge(id)` marks an alarm as seen by an operator.
3. **Resolved**: `resolve(id)` moves an alarm to history (auto-acknowledges).

History auto-trims to `max_history` entries, keeping the most recent.

#### `dashboard.rs` — Orchestrator

`Dashboard` ties everything together:

- **Room navigation**: `next_room()`, `prev_room()`, `select_room(idx)` — wraps around.
- **Scrolling**: `scroll_up()`, `scroll_down()` — for history navigation.
- **Running state**: `set_running(bool)`, `is_running()`.
- **Rendering**: `render_full()` → complete ANSI-encoded string.

Keyboard mapping:
- `Tab` / `Shift+Tab` — Switch rooms.
- `j` / `k` — Scroll.
- `a` — Acknowledge alarm.
- `q` — Quit.

### Design Decisions

**Why no TUI library?** Three reasons:
1. **Portability** — Works in any ANSI-capable terminal. That's all of them.
2. **Testability** — Render to String, assert on contents. No terminal needed.
3. **Simplicity** — ~500 lines of rendering code vs. a dependency tree.

**Why String-based rendering?** The dashboard returns `String` from `render_full()`. This means:
- Unit tests can assert on output without a TTY.
- Output can be logged, piped, or sent over a network.
- The main loop just does `print!("{}", dash.render_full())`.

### Extension Points

#### Custom Panel

Implement a new panel module and add it to `Dashboard::render_full()`:

```rust
pub struct MyPanel { /* ... */ }
impl MyPanel {
    pub fn render(&self, width: usize) -> Vec<String> { /* ... */ }
}
```

#### Custom Color Scheme

Modify `render.rs` to change ANSI codes, or add a theme system:

```rust
pub struct Theme {
    pub header_color: Color,
    pub healthy_color: Color,
    pub warning_color: Color,
    pub critical_color: Color,
}
```

#### Live Data Feed

Replace `main.rs`'s mock data with a real connection to `plato-fleet-manager`:

```rust
// In your render loop:
loop {
    let room_states = fleet_manager.get_room_states().await;
    dash.update_rooms(room_states);
    print!("{}", dash.render_full());
    tokio::time::sleep(Duration::from_millis(1000)).await;
}
```

### Testing Strategy

```bash
cargo test  # 50 tests covering all modules
```

Tests are structured per module:
- **sparkline**: Rendering, edge cases, trend arrows, min/max.
- **room_panel**: Rendering with various alarm states.
- **fleet_panel**: Health aggregation, stats, render output.
- **alarm_panel**: Add, resolve, acknowledge, trim, render.
- **dashboard**: Room switching, scrolling, empty fleet, render_full.

All tests render to String and assert on contents — no terminal required.

### Contributing

1. Keep it dependency-free (std only).
2. All rendering returns `String` or `Vec<String>`.
3. Add tests for any new visual elements.
4. Keep ANSI codes in `render.rs` — don't scatter escape sequences.
5. Run `cargo test` before submitting.
