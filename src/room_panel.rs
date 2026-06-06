//! RoomPanel: single room display with sensor values, alarms, and sparklines.

use crate::render::{Color, Render};
use crate::sparkline::Sparkline;

/// Severity level for alarms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmSeverity {
    Clear,
    Warning,
    Critical,
}

impl AlarmSeverity {
    /// Return the emoji indicator for this severity.
    pub fn emoji(&self) -> &'static str {
        match self {
            AlarmSeverity::Clear => "🟢",
            AlarmSeverity::Warning => "🟡",
            AlarmSeverity::Critical => "🔴",
        }
    }

    /// Return the display color for this severity.
    pub fn color(&self) -> Color {
        match self {
            AlarmSeverity::Clear => Color::Green,
            AlarmSeverity::Warning => Color::Yellow,
            AlarmSeverity::Critical => Color::Red,
        }
    }
}

/// A single sensor reading.
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub alarm: AlarmSeverity,
}

/// The state of a single room.
#[derive(Debug, Clone)]
pub struct RoomState {
    pub name: String,
    pub tick_rate_ms: u64,
    pub last_tick_age_ms: u64,
    pub sensors: Vec<SensorReading>,
    pub history: Vec<Vec<f64>>,
    pub online: bool,
}

/// A panel that renders a single room's status.
#[derive(Debug, Clone)]
pub struct RoomPanel {
    state: RoomState,
}

impl RoomPanel {
    /// Create a new room panel from the given room state.
    pub fn new(state: RoomState) -> Self {
        Self { state }
    }

    /// Get a reference to the room state.
    pub fn state(&self) -> &RoomState {
        &self.state
    }

    /// Render the room panel as a vector of lines.
    pub fn render(&self, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        use crate::render::box_drawing as bd;
        let inner = width.saturating_sub(2);

        // Title bar
        let status_icon = if self.state.online { "●" } else { "○" };
        let status_color = if self.state.online { Color::Green } else { Color::Red };
        let title = format!(
            " {} {} (tick: {}ms, age: {}ms)",
            status_icon, self.state.name, self.state.tick_rate_ms, self.state.last_tick_age_ms
        );
        let colored_title = Render::colorize(&title, status_color);

        lines.push(format!(
            "{}{}{}",
            bd::TOP_LEFT,
            Render::pad(
                &format!(" {} {} (tick: {}ms, age: {}ms) ", status_icon, self.state.name, self.state.tick_rate_ms, self.state.last_tick_age_ms),
                inner
            ),
            bd::TOP_RIGHT
        ));

        // Sensor readings
        for (i, sensor) in self.state.sensors.iter().enumerate() {
            let alarm_emoji = sensor.alarm.emoji();
            let sensor_line = format!(
                " {} {}  {:.1} {}",
                alarm_emoji, sensor.name, sensor.value, sensor.unit
            );
            let colored = if sensor.alarm != AlarmSeverity::Clear {
                Render::colorize(&sensor_line, sensor.alarm.color())
            } else {
                sensor_line.clone()
            };
            lines.push(format!(
                "{}{}{}",
                bd::VERTICAL,
                Render::pad(&sensor_line, inner),
                bd::VERTICAL
            ));

            // Sparkline for this sensor if history available
            if let Some(history) = self.state.history.get(i) {
                if !history.is_empty() {
                    let sp = Sparkline::new(history.clone());
                    let spark_line = format!(
                        "   {} {} [{:.1}..{:.1}]",
                        sp.render(),
                        sp.trend_arrow(),
                        sp.min(),
                        sp.max()
                    );
                    lines.push(format!(
                        "{}{}{}",
                        bd::VERTICAL,
                        Render::pad(&spark_line, inner),
                        bd::VERTICAL
                    ));
                }
            }
        }

        // Bottom border
        lines.push(format!(
            "{}{}{}",
            bd::BOTTOM_LEFT,
            Render::horizontal_line(inner),
            bd::BOTTOM_RIGHT
        ));

        lines
    }

    /// Get the current alarm indicators for all sensors.
    pub fn alarm_indicators(&self) -> Vec<(String, AlarmSeverity)> {
        self.state
            .sensors
            .iter()
            .map(|s| (s.name.clone(), s.alarm))
            .collect()
    }

    /// Get formatted sensor display strings.
    pub fn sensor_display(&self) -> Vec<String> {
        self.state
            .sensors
            .iter()
            .map(|s| format!("{}: {:.1} {}", s.name, s.value, s.unit))
            .collect()
    }

    /// Check if room has any active alarms.
    pub fn has_alarms(&self) -> bool {
        self.state
            .sensors
            .iter()
            .any(|s| s.alarm != AlarmSeverity::Clear)
    }

    /// Get the highest alarm severity in the room.
    pub fn highest_alarm(&self) -> AlarmSeverity {
        self.state
            .sensors
            .iter()
            .map(|s| s.alarm)
            .max_by_key(|s| match s {
                AlarmSeverity::Clear => 0,
                AlarmSeverity::Warning => 1,
                AlarmSeverity::Critical => 2,
            })
            .unwrap_or(AlarmSeverity::Clear)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_room(name: &str, online: bool) -> RoomState {
        RoomState {
            name: name.to_string(),
            tick_rate_ms: 1000,
            last_tick_age_ms: 200,
            sensors: vec![
                SensorReading {
                    name: "Temp".to_string(),
                    value: 22.5,
                    unit: "°C".to_string(),
                    alarm: AlarmSeverity::Clear,
                },
                SensorReading {
                    name: "Humidity".to_string(),
                    value: 45.0,
                    unit: "%".to_string(),
                    alarm: AlarmSeverity::Clear,
                },
            ],
            history: vec![
                vec![20.0, 21.0, 22.0, 22.5],
                vec![40.0, 42.0, 44.0, 45.0],
            ],
            online,
        }
    }

    #[test]
    fn test_create_from_mock_state() {
        let state = mock_room("Engine Room", true);
        let panel = RoomPanel::new(state);
        assert_eq!(panel.state().name, "Engine Room");
        assert_eq!(panel.state().sensors.len(), 2);
    }

    #[test]
    fn test_alarm_indicators() {
        let mut state = mock_room("Lab", true);
        state.sensors[0].alarm = AlarmSeverity::Warning;
        let panel = RoomPanel::new(state);
        let indicators = panel.alarm_indicators();
        assert_eq!(indicators[0].1, AlarmSeverity::Warning);
        assert_eq!(indicators[1].1, AlarmSeverity::Clear);
    }

    #[test]
    fn test_sensor_display() {
        let state = mock_room("Bridge", true);
        let panel = RoomPanel::new(state);
        let display = panel.sensor_display();
        assert_eq!(display.len(), 2);
        assert!(display[0].contains("Temp"));
        assert!(display[0].contains("22.5"));
        assert!(display[1].contains("45.0"));
    }

    #[test]
    fn test_has_alarms() {
        let clear_state = mock_room("A", true);
        let panel_clear = RoomPanel::new(clear_state);
        assert!(!panel_clear.has_alarms());

        let mut alarm_state = mock_room("B", true);
        alarm_state.sensors[0].alarm = AlarmSeverity::Critical;
        let panel_alarm = RoomPanel::new(alarm_state);
        assert!(panel_alarm.has_alarms());
    }

    #[test]
    fn test_highest_alarm() {
        let mut state = mock_room("C", true);
        state.sensors[0].alarm = AlarmSeverity::Warning;
        state.sensors[1].alarm = AlarmSeverity::Critical;
        let panel = RoomPanel::new(state);
        assert_eq!(panel.highest_alarm(), AlarmSeverity::Critical);
    }

    #[test]
    fn test_render_produces_lines() {
        let state = mock_room("D", true);
        let panel = RoomPanel::new(state);
        let lines = panel.render(60);
        assert!(!lines.is_empty());
        // First line should start with top-left box char
        assert!(lines[0].starts_with('┌'));
    }

    #[test]
    fn test_offline_room() {
        let state = mock_room("Offline", false);
        let panel = RoomPanel::new(state);
        assert!(!panel.state().online);
        let lines = panel.render(60);
        assert!(!lines.is_empty());
    }
}
