//! Dashboard: top-level terminal UI with render loop and keyboard input.

use crate::alarm_panel::AlarmPanel;
use crate::fleet_panel::FleetPanel;
use crate::render::{box_drawing, Color, Render};
use crate::room_panel::{AlarmSeverity, RoomPanel, RoomState, SensorReading};

/// Configuration for a fleet of rooms.
#[derive(Debug, Clone)]
pub struct FleetConfig {
    pub rooms: Vec<RoomState>,
    pub refresh_rate_ms: u64,
}

impl FleetConfig {
    /// Create a fleet config with the given rooms and default 1Hz refresh.
    pub fn new(rooms: Vec<RoomState>) -> Self {
        Self {
            rooms,
            refresh_rate_ms: 1000,
        }
    }
}

/// The top-level dashboard.
#[derive(Debug, Clone)]
pub struct Dashboard {
    rooms: Vec<RoomState>,
    alarm_panel: AlarmPanel,
    selected_room: usize,
    refresh_rate_ms: u64,
    width: usize,
    height: usize,
    running: bool,
    scroll_offset: usize,
}

impl Dashboard {
    /// Create a new dashboard from a fleet config.
    pub fn new(config: FleetConfig) -> Self {
        let alarm_panel = AlarmPanel::new(50);
        Self {
            rooms: config.rooms,
            alarm_panel,
            selected_room: 0,
            refresh_rate_ms: config.refresh_rate_ms,
            width: 80,
            height: 24,
            running: false,
            scroll_offset: 0,
        }
    }

    /// Get the currently selected room index.
    pub fn selected_room(&self) -> usize {
        self.selected_room
    }

    /// Switch to the next room.
    pub fn next_room(&mut self) {
        if !self.rooms.is_empty() {
            self.selected_room = (self.selected_room + 1) % self.rooms.len();
        }
    }

    /// Switch to the previous room.
    pub fn prev_room(&mut self) {
        if !self.rooms.is_empty() {
            self.selected_room = (self.selected_room + self.rooms.len() - 1) % self.rooms.len();
        }
    }

    /// Switch to a specific room by index.
    pub fn select_room(&mut self, index: usize) {
        if index < self.rooms.len() {
            self.selected_room = index;
        }
    }

    /// Get a reference to the alarm panel.
    pub fn alarm_panel(&self) -> &AlarmPanel {
        &self.alarm_panel
    }

    /// Get a mutable reference to the alarm panel.
    pub fn alarm_panel_mut(&mut self) -> &mut AlarmPanel {
        &mut self.alarm_panel
    }

    /// Get the number of rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Scroll history up.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll history down.
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Set the running state.
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    /// Check if the dashboard is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Render the full dashboard to a string.
    pub fn render_full(&self) -> String {
        let mut output = String::new();

        output.push_str(&Render::clear_screen());

        // Header
        let header = format!(" ╔══ PLATO DASHBOARD ══╗ [{} rooms, {}Hz refresh]",
            self.rooms.len(),
            1000 / self.refresh_rate_ms.max(1)
        );
        output.push_str(&Render::colorize(&header, Color::Cyan));
        output.push('\n');
        output.push_str(&Render::horizontal_line(self.width));
        output.push('\n');

        // Selected room panel
        if !self.rooms.is_empty() {
            let room = &self.rooms[self.selected_room];
            let panel = RoomPanel::new(room.clone());
            for line in panel.render(self.width) {
                output.push_str(&line);
                output.push('\n');
            }
        }

        output.push('\n');

        // Fleet overview
        let fleet = FleetPanel::new(self.rooms.clone());
        for line in fleet.render(self.width) {
            output.push_str(&line);
            output.push('\n');
        }

        output.push('\n');

        // Alarm panel
        for line in self.alarm_panel.render(self.width) {
            output.push_str(&line);
            output.push('\n');
        }

        // Footer
        output.push('\n');
        let footer = " [Tab] Next room  [Shift+Tab] Prev  [j/k] Scroll  [a] Ack  [q] Quit";
        output.push_str(&Render::colorize(footer, Color::Blue));
        output.push('\n');
        output.push_str(&format!(
            " Room {}/{}",
            self.selected_room + 1,
            self.rooms.len().max(1)
        ));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_room(name: &str) -> RoomState {
        RoomState {
            name: name.to_string(),
            tick_rate_ms: 1000,
            last_tick_age_ms: 200,
            sensors: vec![SensorReading {
                name: "Temp".to_string(),
                value: 22.5,
                unit: "°C".to_string(),
                alarm: AlarmSeverity::Clear,
            }],
            history: vec![vec![20.0, 21.0, 22.0, 22.5]],
            online: true,
        }
    }

    fn mock_config(rooms: usize) -> FleetConfig {
        let states: Vec<RoomState> = (0..rooms)
            .map(|i| mock_room(&format!("Room-{}", i)))
            .collect();
        FleetConfig::new(states)
    }

    #[test]
    fn test_create_from_fleet_config() {
        let config = mock_config(3);
        let dash = Dashboard::new(config);
        assert_eq!(dash.room_count(), 3);
        assert_eq!(dash.selected_room(), 0);
    }

    #[test]
    fn test_room_switching_next() {
        let config = mock_config(3);
        let mut dash = Dashboard::new(config);
        assert_eq!(dash.selected_room(), 0);
        dash.next_room();
        assert_eq!(dash.selected_room(), 1);
        dash.next_room();
        assert_eq!(dash.selected_room(), 2);
        dash.next_room();
        assert_eq!(dash.selected_room(), 0); // wraps
    }

    #[test]
    fn test_room_switching_prev() {
        let config = mock_config(3);
        let mut dash = Dashboard::new(config);
        dash.prev_room();
        assert_eq!(dash.selected_room(), 2); // wraps backwards
        dash.prev_room();
        assert_eq!(dash.selected_room(), 1);
    }

    #[test]
    fn test_select_room() {
        let config = mock_config(5);
        let mut dash = Dashboard::new(config);
        dash.select_room(3);
        assert_eq!(dash.selected_room(), 3);
        dash.select_room(99); // out of bounds, no change
        assert_eq!(dash.selected_room(), 3);
    }

    #[test]
    fn test_running_state() {
        let config = mock_config(1);
        let mut dash = Dashboard::new(config);
        assert!(!dash.is_running());
        dash.set_running(true);
        assert!(dash.is_running());
    }

    #[test]
    fn test_render_full() {
        let config = mock_config(3);
        let dash = Dashboard::new(config);
        let output = dash.render_full();
        assert!(output.contains("PLATO DASHBOARD"));
        assert!(output.contains("Room-0"));
        assert!(output.contains("Fleet Overview"));
    }

    #[test]
    fn test_scroll() {
        let config = mock_config(1);
        let mut dash = Dashboard::new(config);
        dash.scroll_up();
        assert_eq!(dash.scroll_offset, 1);
        dash.scroll_up();
        assert_eq!(dash.scroll_offset, 2);
        dash.scroll_down();
        assert_eq!(dash.scroll_offset, 1);
        dash.scroll_down();
        assert_eq!(dash.scroll_offset, 0);
        dash.scroll_down(); // saturating
        assert_eq!(dash.scroll_offset, 0);
    }

    #[test]
    fn test_empty_fleet() {
        let config = FleetConfig::new(vec![]);
        let dash = Dashboard::new(config);
        assert_eq!(dash.room_count(), 0);
        let output = dash.render_full();
        assert!(output.contains("0 rooms"));
    }
}
