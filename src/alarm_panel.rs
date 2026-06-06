//! AlarmPanel: alarm log, history, and acknowledgment tracking.

use crate::render::{box_drawing, Color, Render};
use crate::room_panel::AlarmSeverity;

/// An alarm event with metadata.
#[derive(Debug, Clone)]
pub struct AlarmEvent {
    pub id: usize,
    pub room_name: String,
    pub sensor_name: String,
    pub severity: AlarmSeverity,
    pub message: String,
    pub timestamp_secs: u64,
    pub acknowledged: bool,
}

/// A panel showing active alarms and history.
#[derive(Debug, Clone)]
pub struct AlarmPanel {
    active: Vec<AlarmEvent>,
    history: Vec<AlarmEvent>,
    max_history: usize,
}

impl AlarmPanel {
    /// Create a new alarm panel with the given maximum history size.
    pub fn new(max_history: usize) -> Self {
        Self {
            active: Vec::new(),
            history: Vec::new(),
            max_history,
        }
    }

    /// Add an active alarm.
    pub fn add_alarm(&mut self, event: AlarmEvent) {
        self.active.push(event);
    }

    /// Resolve an alarm by ID, moving it to history.
    pub fn resolve(&mut self, id: usize) {
        if let Some(pos) = self.active.iter().position(|a| a.id == id) {
            let mut event = self.active.remove(pos);
            event.acknowledged = true;
            self.history.push(event);
            // Trim history
            while self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
    }

    /// Acknowledge an active alarm by ID.
    pub fn acknowledge(&mut self, id: usize) {
        if let Some(event) = self.active.iter_mut().find(|a| a.id == id) {
            event.acknowledged = true;
        }
    }

    /// Get active alarms.
    pub fn active_alarms(&self) -> &[AlarmEvent] {
        &self.active
    }

    /// Get alarm history.
    pub fn alarm_history(&self) -> &[AlarmEvent] {
        &self.history
    }

    /// Count unacknowledged active alarms.
    pub fn unacknowledged_count(&self) -> usize {
        self.active.iter().filter(|a| !a.acknowledged).count()
    }

    /// Render the alarm panel.
    pub fn render(&self, width: usize) -> Vec<String> {
        use crate::render::box_drawing as bd;
        let inner = width.saturating_sub(2);
        let mut lines = Vec::new();

        // Title
        let ack_count = self.unacknowledged_count();
        let title = format!(" Alarms ({} active, {} unacked) ", self.active.len(), ack_count);
        lines.push(format!(
            "{}{}{}",
            bd::TOP_LEFT,
            Render::pad(&title, inner),
            bd::TOP_RIGHT
        ));

        if self.active.is_empty() {
            lines.push(format!(
                "{}{}{}",
                bd::VERTICAL,
                Render::pad("  No active alarms", inner),
                bd::VERTICAL
            ));
        } else {
            // Active alarms
            for event in &self.active {
                let ack_str = if event.acknowledged { "✓" } else { "!" };
                let line = format!(
                    " {} {} [{}] {} > {} ({})",
                    event.severity.emoji(),
                    ack_str,
                    event.id,
                    event.room_name,
                    event.sensor_name,
                    event.message
                );
                lines.push(format!(
                    "{}{}{}",
                    bd::VERTICAL,
                    Render::pad(&line, inner),
                    bd::VERTICAL
                ));
            }
        }

        // Separator
        lines.push(format!(
            "{}{}{}",
            bd::T_RIGHT,
            Render::horizontal_line(inner),
            bd::T_LEFT
        ));

        // History
        let hist_title = format!(" History (last {}) ", self.history.len());
        lines.push(format!(
            "{}{}{}",
            bd::VERTICAL,
            Render::pad(&hist_title, inner),
            bd::VERTICAL
        ));

        for event in &self.history {
            let line = format!(
                "   [{}] {} > {} ✓ @ t={}",
                event.id, event.room_name, event.sensor_name, event.timestamp_secs
            );
            lines.push(format!(
                "{}{}{}",
                bd::VERTICAL,
                Render::pad(&line, inner),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_alarm(id: usize, room: &str, severity: AlarmSeverity, acked: bool) -> AlarmEvent {
        AlarmEvent {
            id,
            room_name: room.to_string(),
            sensor_name: "Temp".to_string(),
            severity,
            message: "High temperature".to_string(),
            timestamp_secs: 1000 + id as u64,
            acknowledged: acked,
        }
    }

    #[test]
    fn test_active_alarms() {
        let mut panel = AlarmPanel::new(100);
        panel.add_alarm(make_alarm(1, "A", AlarmSeverity::Warning, false));
        panel.add_alarm(make_alarm(2, "B", AlarmSeverity::Critical, false));
        assert_eq!(panel.active_alarms().len(), 2);
    }

    #[test]
    fn test_alarm_history() {
        let mut panel = AlarmPanel::new(100);
        panel.add_alarm(make_alarm(1, "A", AlarmSeverity::Warning, false));
        panel.resolve(1);
        assert_eq!(panel.active_alarms().len(), 0);
        assert_eq!(panel.alarm_history().len(), 1);
        assert!(panel.alarm_history()[0].acknowledged);
    }

    #[test]
    fn test_acknowledgment_tracking() {
        let mut panel = AlarmPanel::new(100);
        panel.add_alarm(make_alarm(1, "A", AlarmSeverity::Critical, false));
        panel.add_alarm(make_alarm(2, "B", AlarmSeverity::Warning, false));
        assert_eq!(panel.unacknowledged_count(), 2);

        panel.acknowledge(1);
        assert_eq!(panel.unacknowledged_count(), 1);

        // Acknowledge non-existent ID is a no-op
        panel.acknowledge(999);
        assert_eq!(panel.unacknowledged_count(), 1);
    }

    #[test]
    fn test_history_trimming() {
        let mut panel = AlarmPanel::new(3);
        for i in 0..5 {
            panel.add_alarm(make_alarm(i, "X", AlarmSeverity::Warning, false));
            panel.resolve(i);
        }
        assert_eq!(panel.alarm_history().len(), 3);
        // Should keep the most recent 3
        assert_eq!(panel.alarm_history()[0].id, 2);
        assert_eq!(panel.alarm_history()[2].id, 4);
    }

    #[test]
    fn test_render() {
        let mut panel = AlarmPanel::new(100);
        panel.add_alarm(make_alarm(1, "A", AlarmSeverity::Warning, false));
        let lines = panel.render(70);
        assert!(!lines.is_empty());
        assert!(lines[0].starts_with('┌'));
    }

    #[test]
    fn test_render_no_alarms() {
        let panel = AlarmPanel::new(100);
        let lines = panel.render(70);
        assert!(lines.iter().any(|l| l.contains("No active alarms")));
    }

    #[test]
    fn test_resolve_nonexistent() {
        let mut panel = AlarmPanel::new(100);
        panel.add_alarm(make_alarm(1, "A", AlarmSeverity::Warning, false));
        panel.resolve(999); // should be a no-op
        assert_eq!(panel.active_alarms().len(), 1);
    }
}
