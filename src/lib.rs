//! Fleet dashboard rendering for PLATO nervous system.
//!
//! Aggregates room states, autonomy metrics, alerts, and coordination events
//! into a structured dashboard renderable as text, JSON, or HTML.

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Severity ────────────────────────────────────────────────────────────────

/// Alert severity levels, ordered Info < Warning < Critical < Emergency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl Severity {
    /// Emoji icon for terminal display.
    pub fn icon(&self) -> &'static str {
        match self {
            Severity::Info => "ℹ️",
            Severity::Warning => "⚠️",
            Severity::Critical => "🔴",
            Severity::Emergency => "🚨",
        }
    }

    /// Short label.
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Info => "INFO",
            Severity::Warning => "WARN",
            Severity::Critical => "CRIT",
            Severity::Emergency => "EMRG",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ── AlertSummary ────────────────────────────────────────────────────────────

/// A single alert associated with a room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub room_id: String,
    pub alert_type: String,
    pub message: String,
    pub severity: Severity,
    pub timestamp: u64,
    pub acknowledged: bool,
}

impl AlertSummary {
    pub fn new(
        room_id: impl Into<String>,
        alert_type: impl Into<String>,
        message: impl Into<String>,
        severity: Severity,
        timestamp: u64,
    ) -> Self {
        Self {
            room_id: room_id.into(),
            alert_type: alert_type.into(),
            message: message.into(),
            severity,
            timestamp,
            acknowledged: false,
        }
    }

    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

// ── RoomStatus ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomStatus {
    Ok,
    Warning,
    Critical,
    Offline,
}

impl RoomStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            RoomStatus::Ok => "✅",
            RoomStatus::Warning => "⚠",
            RoomStatus::Critical => "🔴",
            RoomStatus::Offline => "⬛",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RoomStatus::Ok => "OK",
            RoomStatus::Warning => "WARN",
            RoomStatus::Critical => "CRIT",
            RoomStatus::Offline => "OFF",
        }
    }
}

impl fmt::Display for RoomStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ── RoomType ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomType {
    Engine,
    Bridge,
    Galley,
    Hold,
    Cabin,
    Lab,
    Corridor,
    Other(String),
}

// ── RoomSummary ─────────────────────────────────────────────────────────────

/// Aggregated state of a single room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSummary {
    pub room_id: String,
    pub room_type: RoomType,
    pub status: RoomStatus,
    pub autonomy_pct: f64,
    pub health_score: f64,
    pub last_reading_ago_secs: u64,
    pub active_alerts: usize,
}

impl RoomSummary {
    pub fn new(
        room_id: impl Into<String>,
        room_type: RoomType,
        status: RoomStatus,
        autonomy_pct: f64,
        health_score: f64,
        last_reading_ago_secs: u64,
    ) -> Self {
        Self {
            room_id: room_id.into(),
            room_type,
            status,
            autonomy_pct,
            health_score,
            last_reading_ago_secs,
            active_alerts: 0,
        }
    }
}

// ── FleetSummary ────────────────────────────────────────────────────────────

/// Aggregate fleet-level statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSummary {
    pub total_rooms: usize,
    pub rooms_ok: usize,
    pub rooms_warning: usize,
    pub rooms_critical: usize,
    pub rooms_offline: usize,
    pub fleet_autonomy_pct: f64,
    pub total_alerts: usize,
    pub avg_health: f64,
}

impl FleetSummary {
    /// Compute fleet summary from room summaries and alert count.
    pub fn from_rooms(rooms: &[RoomSummary], total_alerts: usize) -> Self {
        let total_rooms = rooms.len();
        let rooms_ok = rooms.iter().filter(|r| r.status == RoomStatus::Ok).count();
        let rooms_warning = rooms.iter().filter(|r| r.status == RoomStatus::Warning).count();
        let rooms_critical = rooms.iter().filter(|r| r.status == RoomStatus::Critical).count();
        let rooms_offline = rooms.iter().filter(|r| r.status == RoomStatus::Offline).count();

        let fleet_autonomy_pct = if total_rooms > 0 {
            rooms.iter().map(|r| r.autonomy_pct).sum::<f64>() / total_rooms as f64
        } else {
            0.0
        };

        let avg_health = if total_rooms > 0 {
            rooms.iter().map(|r| r.health_score).sum::<f64>() / total_rooms as f64
        } else {
            0.0
        };

        Self {
            total_rooms,
            rooms_ok,
            rooms_warning,
            rooms_critical,
            rooms_offline,
            fleet_autonomy_pct,
            total_alerts,
            avg_health,
        }
    }
}

// ── DashboardConfig ─────────────────────────────────────────────────────────

/// Dashboard configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub fleet_name: String,
    pub refresh_interval_secs: u64,
    pub alert_ttl_secs: u64,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            fleet_name: "F/V AURORA".into(),
            refresh_interval_secs: 30,
            alert_ttl_secs: 3600,
        }
    }
}

impl DashboardConfig {
    pub fn new(fleet_name: impl Into<String>) -> Self {
        Self {
            fleet_name: fleet_name.into(),
            ..Default::default()
        }
    }

    /// Validate config. Returns `Ok(())` or an error message.
    pub fn validate(&self) -> Result<(), String> {
        if self.fleet_name.trim().is_empty() {
            return Err("fleet_name must not be empty".into());
        }
        if self.refresh_interval_secs == 0 {
            return Err("refresh_interval_secs must be > 0".into());
        }
        if self.alert_ttl_secs == 0 {
            return Err("alert_ttl_secs must be > 0".into());
        }
        Ok(())
    }
}

// ── Dashboard ───────────────────────────────────────────────────────────────

/// Top-level dashboard container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub fleet_name: String,
    pub timestamp: u64,
    pub rooms: Vec<RoomSummary>,
    pub fleet_summary: FleetSummary,
    pub active_alerts: Vec<AlertSummary>,
}

impl Dashboard {
    /// Build a dashboard from config, rooms, alerts, and a unix timestamp.
    pub fn new(
        config: &DashboardConfig,
        rooms: Vec<RoomSummary>,
        active_alerts: Vec<AlertSummary>,
        timestamp: u64,
    ) -> Self {
        let fleet_summary = FleetSummary::from_rooms(&rooms, active_alerts.len());
        Self {
            fleet_name: config.fleet_name.clone(),
            timestamp,
            rooms,
            fleet_summary,
            active_alerts,
        }
    }

    /// Render dashboard as pretty-printed JSON.
    pub fn render_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    /// Render dashboard as a formatted ASCII table for terminal display.
    pub fn render_text(&self) -> String {
        let mut out = String::with_capacity(2048);
        let ts_str = format_unix_timestamp(self.timestamp);

        // Header
        let title = format!("  {} — Fleet Dashboard    {}  ", self.fleet_name, ts_str);
        let header_width = 50;
        out.push_str(&format!("╔{}╗\n", "═".repeat(header_width)));
        out.push_str(&format!("║{:^width$}║\n", title, width = header_width));
        out.push_str(&format!("╠{}╣\n", "═".repeat(header_width)));

        // Fleet summary line
        let crit_count = self
            .active_alerts
            .iter()
            .filter(|a| a.severity >= Severity::Critical)
            .count();
        let summary = format!(
            "  Fleet Autonomy: {:.1}%  │  Alerts: {} ({} CRIT)   ",
            self.fleet_summary.fleet_autonomy_pct,
            self.fleet_summary.total_alerts,
            crit_count
        );
        out.push_str(&format!("║{:^width$}║\n", summary, width = header_width));

        // Table header
        out.push_str(&format!(
            "╠════════════╦═══════════╦═══════════╦═════════════╣\n"
        ));
        out.push_str(&format!(
            "║  Room      ║ Status    ║ Autonomy  ║ Health      ║\n"
        ));
        out.push_str(&format!(
            "╠════════════╬═══════════╬═══════════╬═════════════╣\n"
        ));

        // Room rows
        for room in &self.rooms {
            let status_str = format!("{} {}", room.status.icon(), room.status.label());
            out.push_str(&format!(
                "║  {:<10}║ {:<9}║ {:>8.1}%  ║ {:>11.2} ║\n",
                room.room_id, status_str, room.autonomy_pct, room.health_score
            ));
        }

        out.push_str(&format!(
            "╚════════════╩═══════════╩═══════════╩═════════════╝\n"
        ));
        out
    }

    /// Render dashboard as minimal HTML with inline CSS (dark theme).
    pub fn render_html(&self) -> String {
        let ts_str = format_unix_timestamp(self.timestamp);
        let crit_count = self
            .active_alerts
            .iter()
            .filter(|a| a.severity >= Severity::Critical)
            .count();

        let mut html = String::with_capacity(4096);
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
        html.push_str(&format!(
            "<title>{} — Fleet Dashboard</title>\n",
            self.fleet_name
        ));
        html.push_str("<style>\n");
        html.push_str("body{font-family:monospace;background:#0d1117;color:#c9d1d9;margin:2em;}\n");
        html.push_str("h1{color:#58a6ff;border-bottom:1px solid #30363d;padding-bottom:.5em;}\n");
        html.push_str("h2{color:#8b949e;margin-top:1.5em;}\n");
        html.push_str(
            "table{border-collapse:collapse;width:100%;margin-top:1em;}\n",
        );
        html.push_str("th,td{border:1px solid #30363d;padding:8px 12px;text-align:left;}\n");
        html.push_str("th{background:#161b22;color:#58a6ff;}\n");
        html.push_str("tr:nth-child(even){background:#161b22;}\n");
        html.push_str(".ok{color:#3fb950;}.warn{color:#d29922;}.crit{color:#f85149;}.off{color:#484f58;}\n");
        html.push_str(".summary{display:flex;gap:2em;margin:1em 0;flex-wrap:wrap;}\n");
        html.push_str(".stat{background:#161b22;border:1px solid #30363d;border-radius:6px;padding:1em 1.5em;}\n");
        html.push_str(".stat .label{color:#8b949e;font-size:.85em;}\n");
        html.push_str(".stat .value{color:#58a6ff;font-size:1.4em;font-weight:bold;}\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str(&format!(
            "<h1>{} — Fleet Dashboard</h1>\n",
            self.fleet_name
        ));
        html.push_str(&format!("<p>Generated: {}</p>\n", ts_str));

        // Summary cards
        html.push_str("<div class=\"summary\">\n");
        html.push_str(&stat_card(
            "Fleet Autonomy",
            &format!("{:.1}%", self.fleet_summary.fleet_autonomy_pct),
        ));
        html.push_str(&stat_card(
            "Total Alerts",
            &format!(
                "{} ({} CRIT)",
                self.fleet_summary.total_alerts, crit_count
            ),
        ));
        html.push_str(&stat_card(
            "Avg Health",
            &format!("{:.2}", self.fleet_summary.avg_health),
        ));
        html.push_str(&stat_card(
            "Rooms",
            &format!(
                "{} OK / {} WARN / {} CRIT / {} OFF",
                self.fleet_summary.rooms_ok,
                self.fleet_summary.rooms_warning,
                self.fleet_summary.rooms_critical,
                self.fleet_summary.rooms_offline
            ),
        ));
        html.push_str("</div>\n");

        // Rooms table
        html.push_str("<h2>Room Status</h2>\n<table>\n");
        html.push_str("<tr><th>Room</th><th>Status</th><th>Autonomy</th><th>Health</th><th>Last Reading</th><th>Alerts</th></tr>\n");
        for room in &self.rooms {
            let css_class = match room.status {
                RoomStatus::Ok => "ok",
                RoomStatus::Warning => "warn",
                RoomStatus::Critical => "crit",
                RoomStatus::Offline => "off",
            };
            html.push_str(&format!(
                "<tr><td>{}</td><td class=\"{}\">{} {}</td><td>{:.1}%</td><td>{:.2}</td><td>{}s ago</td><td>{}</td></tr>\n",
                room.room_id,
                css_class,
                room.status.icon(),
                room.status.label(),
                room.autonomy_pct,
                room.health_score,
                room.last_reading_ago_secs,
                room.active_alerts,
            ));
        }
        html.push_str("</table>\n");

        // Alerts table
        if !self.active_alerts.is_empty() {
            html.push_str("<h2>Active Alerts</h2>\n<table>\n");
            html.push_str("<tr><th>Room</th><th>Type</th><th>Severity</th><th>Message</th><th>Ack</th></tr>\n");
            for alert in &self.active_alerts {
                let css_class = match alert.severity {
                    Severity::Info => "ok",
                    Severity::Warning => "warn",
                    Severity::Critical | Severity::Emergency => "crit",
                };
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td class=\"{}\">{} {}</td><td>{}</td><td>{}</td></tr>\n",
                    alert.room_id,
                    alert.alert_type,
                    css_class,
                    alert.severity.icon(),
                    alert.severity,
                    alert.message,
                    if alert.acknowledged { "✅" } else { "❌" },
                ));
            }
            html.push_str("</table>\n");
        }

        html.push_str("</body>\n</html>\n");
        html
    }

    /// Sort alerts by severity (descending — most severe first).
    pub fn sort_alerts_by_severity(&mut self) {
        self.active_alerts.sort_by(|a, b| b.severity.cmp(&a.severity));
    }
}

fn stat_card(label: &str, value: &str) -> String {
    format!(
        "<div class=\"stat\"><div class=\"label\">{}</div><div class=\"value\">{}</div></div>\n",
        label, value
    )
}

/// Format a unix timestamp as HH:MM:SS. In a real system this would use chrono.
fn format_unix_timestamp(ts: u64) -> String {
    let secs = ts % 86400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rooms() -> Vec<RoomSummary> {
        vec![
            RoomSummary::new("Engine", RoomType::Engine, RoomStatus::Warning, 89.1, 0.72, 5),
            RoomSummary::new("Bridge", RoomType::Bridge, RoomStatus::Ok, 98.4, 0.95, 2),
            RoomSummary::new("Galley", RoomType::Galley, RoomStatus::Ok, 97.2, 0.91, 3),
            RoomSummary::new("Hold", RoomType::Hold, RoomStatus::Critical, 45.3, 0.34, 12),
        ]
    }

    fn sample_alerts() -> Vec<AlertSummary> {
        vec![
            AlertSummary::new("Engine", "overheat", "Engine temp above threshold", Severity::Warning, 1000),
            AlertSummary::new("Hold", "pressure", "Hold pressure critical", Severity::Critical, 1005),
            AlertSummary::new("Bridge", "info", "Navigation update available", Severity::Info, 1010),
        ]
    }

    fn sample_dashboard() -> Dashboard {
        let config = DashboardConfig::new("F/V AURORA");
        Dashboard::new(&config, sample_rooms(), sample_alerts(), 45000)
    }

    // ── Dashboard construction ──────────────────────────────────────────

    #[test]
    fn dashboard_construction() {
        let d = sample_dashboard();
        assert_eq!(d.fleet_name, "F/V AURORA");
        assert_eq!(d.rooms.len(), 4);
        assert_eq!(d.active_alerts.len(), 3);
        assert_eq!(d.timestamp, 45000);
    }

    #[test]
    fn fleet_summary_counts() {
        let d = sample_dashboard();
        let s = &d.fleet_summary;
        assert_eq!(s.total_rooms, 4);
        assert_eq!(s.rooms_ok, 2);
        assert_eq!(s.rooms_warning, 1);
        assert_eq!(s.rooms_critical, 1);
        assert_eq!(s.rooms_offline, 0);
    }

    #[test]
    fn fleet_summary_autonomy_avg() {
        let d = sample_dashboard();
        let expected = (89.1 + 98.4 + 97.2 + 45.3) / 4.0;
        assert!((d.fleet_summary.fleet_autonomy_pct - expected).abs() < 0.01);
    }

    #[test]
    fn fleet_summary_health_avg() {
        let d = sample_dashboard();
        let expected = (0.72 + 0.95 + 0.91 + 0.34) / 4.0;
        assert!((d.fleet_summary.avg_health - expected).abs() < 0.01);
    }

    #[test]
    fn fleet_summary_total_alerts() {
        let d = sample_dashboard();
        assert_eq!(d.fleet_summary.total_alerts, 3);
    }

    // ── AlertSummary ────────────────────────────────────────────────────

    #[test]
    fn alert_creation() {
        let a = AlertSummary::new("R1", "fire", "Fire detected", Severity::Emergency, 42);
        assert_eq!(a.room_id, "R1");
        assert_eq!(a.alert_type, "fire");
        assert!(!a.acknowledged);
    }

    #[test]
    fn alert_acknowledge() {
        let mut a = AlertSummary::new("R1", "test", "msg", Severity::Info, 0);
        assert!(!a.acknowledged);
        a.acknowledge();
        assert!(a.acknowledged);
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Critical);
        assert!(Severity::Critical < Severity::Emergency);
    }

    #[test]
    fn alert_severity_sort() {
        let mut d = sample_dashboard();
        d.sort_alerts_by_severity();
        assert_eq!(d.active_alerts[0].severity, Severity::Critical);
        assert_eq!(d.active_alerts[1].severity, Severity::Warning);
        assert_eq!(d.active_alerts[2].severity, Severity::Info);
    }

    // ── JSON rendering ──────────────────────────────────────────────────

    #[test]
    fn render_json_valid() {
        let d = sample_dashboard();
        let json = d.render_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");
        assert_eq!(parsed["fleet_name"], "F/V AURORA");
        assert_eq!(parsed["rooms"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn render_json_contains_rooms_and_alerts() {
        let d = sample_dashboard();
        let json = d.render_json();
        assert!(json.contains("\"rooms\""));
        assert!(json.contains("\"active_alerts\""));
        assert!(json.contains("\"fleet_summary\""));
    }

    // ── Text rendering ──────────────────────────────────────────────────

    #[test]
    fn render_text_contains_table() {
        let d = sample_dashboard();
        let text = d.render_text();
        assert!(text.contains("╔"));
        assert!(text.contains("╚"));
        assert!(text.contains("Engine"));
        assert!(text.contains("Bridge"));
    }

    #[test]
    fn render_text_shows_fleet_name() {
        let d = sample_dashboard();
        let text = d.render_text();
        assert!(text.contains("F/V AURORA"));
    }

    #[test]
    fn render_text_shows_status_icons() {
        let d = sample_dashboard();
        let text = d.render_text();
        assert!(text.contains("⚠"));
        assert!(text.contains("✅"));
        assert!(text.contains("🔴"));
    }

    // ── HTML rendering ──────────────────────────────────────────────────

    #[test]
    fn render_html_valid_structure() {
        let d = sample_dashboard();
        let html = d.render_html();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("</html>"));
        assert!(html.contains("<table>"));
        assert!(html.contains("</table>"));
    }

    #[test]
    fn render_html_dark_theme() {
        let d = sample_dashboard();
        let html = d.render_html();
        assert!(html.contains("background:#0d1117"));
        assert!(html.contains("color:#c9d1d9"));
    }

    #[test]
    fn render_html_contains_rooms() {
        let d = sample_dashboard();
        let html = d.render_html();
        assert!(html.contains("Engine"));
        assert!(html.contains("Bridge"));
        assert!(html.contains("Galley"));
        assert!(html.contains("Hold"));
    }

    #[test]
    fn render_html_contains_alerts_table() {
        let d = sample_dashboard();
        let html = d.render_html();
        assert!(html.contains("Active Alerts"));
        assert!(html.contains("overheat"));
    }

    // ── Edge cases ──────────────────────────────────────────────────────

    #[test]
    fn empty_fleet() {
        let config = DashboardConfig::new("Empty Fleet");
        let d = Dashboard::new(&config, vec![], vec![], 0);
        assert_eq!(d.fleet_summary.total_rooms, 0);
        assert_eq!(d.fleet_summary.fleet_autonomy_pct, 0.0);
        assert_eq!(d.fleet_summary.avg_health, 0.0);
        let json = d.render_json();
        serde_json::from_str::<serde_json::Value>(&json).unwrap();
    }

    #[test]
    fn all_rooms_critical() {
        let rooms = vec![
            RoomSummary::new("R1", RoomType::Engine, RoomStatus::Critical, 10.0, 0.1, 60),
            RoomSummary::new("R2", RoomType::Hold, RoomStatus::Critical, 5.0, 0.05, 120),
        ];
        let config = DashboardConfig::new("Doomed");
        let d = Dashboard::new(&config, rooms, vec![], 0);
        assert_eq!(d.fleet_summary.rooms_critical, 2);
        assert_eq!(d.fleet_summary.rooms_ok, 0);
        assert!(d.fleet_summary.avg_health < 0.2);
    }

    #[test]
    fn no_alerts() {
        let config = DashboardConfig::new("Quiet");
        let rooms = vec![RoomSummary::new("R1", RoomType::Bridge, RoomStatus::Ok, 99.0, 0.99, 1)];
        let d = Dashboard::new(&config, rooms, vec![], 0);
        assert_eq!(d.active_alerts.len(), 0);
        assert_eq!(d.fleet_summary.total_alerts, 0);
        let html = d.render_html();
        assert!(!html.contains("Active Alerts"));
    }

    #[test]
    fn single_room() {
        let config = DashboardConfig::new("Solo");
        let rooms = vec![RoomSummary::new("Only", RoomType::Other("Custom".into()), RoomStatus::Ok, 100.0, 1.0, 0)];
        let d = Dashboard::new(&config, rooms, vec![], 0);
        assert_eq!(d.fleet_summary.total_rooms, 1);
        assert_eq!(d.fleet_summary.rooms_ok, 1);
        assert!((d.fleet_summary.fleet_autonomy_pct - 100.0).abs() < 0.01);
    }

    // ── Config validation ───────────────────────────────────────────────

    #[test]
    fn config_valid_default() {
        assert!(DashboardConfig::default().validate().is_ok());
    }

    #[test]
    fn config_rejects_empty_name() {
        let mut c = DashboardConfig::default();
        c.fleet_name = "  ".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn config_rejects_zero_refresh() {
        let mut c = DashboardConfig::default();
        c.refresh_interval_secs = 0;
        assert!(c.validate().is_err());
    }

    // ── Serialization round-trip ────────────────────────────────────────

    #[test]
    fn serde_roundtrip_dashboard() {
        let d = sample_dashboard();
        let json = serde_json::to_string(&d).unwrap();
        let d2: Dashboard = serde_json::from_str(&json).unwrap();
        assert_eq!(d2.fleet_name, d.fleet_name);
        assert_eq!(d2.rooms.len(), d.rooms.len());
        assert_eq!(d2.active_alerts.len(), d.active_alerts.len());
    }

    #[test]
    fn serde_roundtrip_severity() {
        for s in [Severity::Info, Severity::Warning, Severity::Critical, Severity::Emergency] {
            let json = serde_json::to_string(&s).unwrap();
            let s2: Severity = serde_json::from_str(&json).unwrap();
            assert_eq!(s, s2);
        }
    }
}
