//! FleetPanel: fleet overview with one line per room and aggregate health.

use crate::render::{box_drawing, Color, Render};
use crate::room_panel::RoomState;

/// Fleet health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleetHealth {
    Green,
    Yellow,
    Red,
}

impl FleetHealth {
    pub fn emoji(&self) -> &'static str {
        match self {
            FleetHealth::Green => "🟢",
            FleetHealth::Yellow => "🟡",
            FleetHealth::Red => "🔴",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            FleetHealth::Green => Color::Green,
            FleetHealth::Yellow => Color::Yellow,
            FleetHealth::Red => Color::Red,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FleetHealth::Green => "HEALTHY",
            FleetHealth::Yellow => "DEGRADED",
            FleetHealth::Red => "CRITICAL",
        }
    }
}

/// Fleet-wide summary statistics.
#[derive(Debug, Clone)]
pub struct FleetStats {
    pub total_rooms: usize,
    pub total_sensors: usize,
    pub total_alarms: usize,
    pub uptime_seconds: u64,
    pub health: FleetHealth,
}

/// A panel showing the fleet overview.
#[derive(Debug, Clone)]
pub struct FleetPanel {
    rooms: Vec<RoomState>,
}

impl FleetPanel {
    /// Create a new fleet panel from room states.
    pub fn new(rooms: Vec<RoomState>) -> Self {
        Self { rooms }
    }

    /// Compute the health of a single room based on its state.
    pub fn room_health(room: &RoomState) -> FleetHealth {
        if !room.online {
            return FleetHealth::Red;
        }
        let has_critical = room.sensors.iter().any(|s| {
            matches!(s.alarm, crate::room_panel::AlarmSeverity::Critical)
        });
        if has_critical {
            return FleetHealth::Red;
        }
        let has_warning = room.sensors.iter().any(|s| {
            matches!(s.alarm, crate::room_panel::AlarmSeverity::Warning)
        });
        if has_warning {
            return FleetHealth::Yellow;
        }
        FleetHealth::Green
    }

    /// Compute the aggregate fleet health.
    pub fn aggregate_health(&self) -> FleetHealth {
        if self.rooms.is_empty() {
            return FleetHealth::Green;
        }
        let healths: Vec<FleetHealth> = self.rooms.iter().map(Self::room_health).collect();
        if healths.iter().any(|h| *h == FleetHealth::Red) {
            FleetHealth::Red
        } else if healths.iter().any(|h| *h == FleetHealth::Yellow) {
            FleetHealth::Yellow
        } else {
            FleetHealth::Green
        }
    }

    /// Compute fleet-wide statistics.
    pub fn stats(&self) -> FleetStats {
        let total_rooms = self.rooms.len();
        let total_sensors: usize = self.rooms.iter().map(|r| r.sensors.len()).sum();
        let total_alarms: usize = self
            .rooms
            .iter()
            .map(|r| {
                r.sensors
                    .iter()
                    .filter(|s| s.alarm != crate::room_panel::AlarmSeverity::Clear)
                    .count()
            })
            .sum();
        FleetStats {
            total_rooms,
            total_sensors,
            total_alarms,
            uptime_seconds: 0,
            health: self.aggregate_health(),
        }
    }

    /// Render the fleet panel as lines.
    pub fn render(&self, width: usize) -> Vec<String> {
        use crate::render::box_drawing as bd;
        let inner = width.saturating_sub(2);
        let mut lines = Vec::new();

        // Title
        let title = format!(" Fleet Overview ");
        lines.push(format!(
            "{}{}{}",
            bd::TOP_LEFT,
            Render::pad(&title, inner),
            bd::TOP_RIGHT
        ));

        // Header
        let header = format!(" {:<20} {:<10} {:<10} {:<12}", "Room", "Health", "Alarms", "Last Tick");
        lines.push(format!(
            "{}{}{}",
            bd::VERTICAL,
            Render::pad(&header, inner),
            bd::VERTICAL
        ));

        // Separator
        lines.push(format!(
            "{}{}{}",
            bd::T_RIGHT,
            Render::horizontal_line(inner),
            bd::T_LEFT
        ));

        // One line per room
        for room in &self.rooms {
            let health = Self::room_health(room);
            let alarm_count: usize = room
                .sensors
                .iter()
                .filter(|s| s.alarm != crate::room_panel::AlarmSeverity::Clear)
                .count();
            let age_str = if room.online {
                format!("{}ms", room.last_tick_age_ms)
            } else {
                "OFFLINE".to_string()
            };
            let row = format!(
                " {:<20} {:<10} {:<10} {:<12}",
                room.name,
                health.label(),
                alarm_count,
                age_str
            );
            let colored_row = if health == FleetHealth::Red {
                Render::colorize(&row, Color::Red)
            } else if health == FleetHealth::Yellow {
                Render::colorize(&row, Color::Yellow)
            } else {
                row.clone()
            };
            lines.push(format!(
                "{}{}{}",
                bd::VERTICAL,
                Render::pad(&row, inner),
                bd::VERTICAL
            ));
        }

        // Separator
        lines.push(format!(
            "{}{}{}",
            bd::T_RIGHT,
            Render::horizontal_line(inner),
            bd::T_LEFT
        ));

        // Aggregate stats
        let stats = self.stats();
        let agg = format!(
            " {} {} | {} rooms, {} sensors, {} alarms",
            stats.health.emoji(),
            stats.health.label(),
            stats.total_rooms,
            stats.total_sensors,
            stats.total_alarms
        );
        lines.push(format!(
            "{}{}{}",
            bd::VERTICAL,
            Render::pad(&agg, inner),
            bd::VERTICAL
        ));

        // Bottom
        lines.push(format!(
            "{}{}{}",
            bd::BOTTOM_LEFT,
            Render::horizontal_line(inner),
            bd::BOTTOM_RIGHT
        ));

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room_panel::{AlarmSeverity, SensorReading};

    fn make_room(name: &str, online: bool, alarms: Vec<AlarmSeverity>) -> RoomState {
        let sensors: Vec<SensorReading> = alarms
            .into_iter()
            .enumerate()
            .map(|(i, alarm)| SensorReading {
                name: format!("S{}", i),
                value: 10.0 + i as f64,
                unit: "°C".to_string(),
                alarm,
            })
            .collect();
        RoomState {
            name: name.to_string(),
            tick_rate_ms: 1000,
            last_tick_age_ms: 200,
            sensors,
            history: vec![],
            online,
        }
    }

    #[test]
    fn test_one_room_healthy() {
        let room = make_room("A", true, vec![AlarmSeverity::Clear]);
        let panel = FleetPanel::new(vec![room]);
        assert_eq!(FleetPanel::room_health(&panel.rooms[0]), FleetHealth::Green);
    }

    #[test]
    fn test_one_room_with_warning() {
        let room = make_room("B", true, vec![AlarmSeverity::Warning]);
        let panel = FleetPanel::new(vec![room]);
        assert_eq!(FleetPanel::room_health(&panel.rooms[0]), FleetHealth::Yellow);
    }

    #[test]
    fn test_one_room_with_alarm() {
        let room = make_room("C", true, vec![AlarmSeverity::Warning, AlarmSeverity::Clear]);
        let panel = FleetPanel::new(vec![room]);
        assert_eq!(FleetPanel::room_health(&panel.rooms[0]), FleetHealth::Yellow);
    }

    #[test]
    fn test_one_room_offline() {
        let room = make_room("D", false, vec![]);
        let panel = FleetPanel::new(vec![room]);
        assert_eq!(FleetPanel::room_health(&panel.rooms[0]), FleetHealth::Red);
    }

    #[test]
    fn test_aggregate_health() {
        let rooms = vec![
            make_room("A", true, vec![AlarmSeverity::Clear]),
            make_room("B", true, vec![AlarmSeverity::Warning]),
        ];
        let panel = FleetPanel::new(rooms);
        assert_eq!(panel.aggregate_health(), FleetHealth::Yellow);
    }

    #[test]
    fn test_aggregate_all_green() {
        let rooms = vec![
            make_room("A", true, vec![AlarmSeverity::Clear]),
            make_room("B", true, vec![AlarmSeverity::Clear]),
        ];
        let panel = FleetPanel::new(rooms);
        assert_eq!(panel.aggregate_health(), FleetHealth::Green);
    }

    #[test]
    fn test_aggregate_one_red() {
        let rooms = vec![
            make_room("A", true, vec![AlarmSeverity::Clear]),
            make_room("B", false, vec![]),
        ];
        let panel = FleetPanel::new(rooms);
        assert_eq!(panel.aggregate_health(), FleetHealth::Red);
    }

    #[test]
    fn test_stats() {
        let rooms = vec![
            make_room("A", true, vec![AlarmSeverity::Clear, AlarmSeverity::Warning]),
            make_room("B", true, vec![AlarmSeverity::Clear]),
        ];
        let panel = FleetPanel::new(rooms);
        let stats = panel.stats();
        assert_eq!(stats.total_rooms, 2);
        assert_eq!(stats.total_sensors, 3);
        assert_eq!(stats.total_alarms, 1);
        assert_eq!(stats.health, FleetHealth::Yellow);
    }

    #[test]
    fn test_render() {
        let rooms = vec![
            make_room("A", true, vec![AlarmSeverity::Clear]),
        ];
        let panel = FleetPanel::new(rooms);
        let lines = panel.render(70);
        assert!(!lines.is_empty());
        assert!(lines[0].starts_with('┌'));
    }
}
