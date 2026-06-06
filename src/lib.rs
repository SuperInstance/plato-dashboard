//! # Plato Dashboard
//!
//! A terminal-based dashboard for monitoring Plato rooms. Connects to a fleet of rooms,
//! displays live sensor values, alarm states, history sparklines, and fleet health —
//! all in the terminal. No GUI, no web browser, just text.
//!
//! Designed for captains on boats, ops in data centers, or anyone who needs room awareness
//! at a glance.
//!
//! ## Modules
//!
//! - [`dashboard`] — Top-level terminal UI with render loop
//! - [`room_panel`] — Single room display with sensors, alarms, sparklines
//! - [`fleet_panel`] — Fleet overview with health aggregation
//! - [`alarm_panel`] — Alarm log, history, and acknowledgment tracking
//! - [`sparkline`] — Numeric series rendered as Unicode sparklines
//! - [`render`] — Terminal rendering utilities (ANSI escape codes, box drawing)

pub mod alarm_panel;
pub mod dashboard;
pub mod fleet_panel;
pub mod render;
pub mod room_panel;
pub mod sparkline;

pub use alarm_panel::{AlarmEvent, AlarmPanel};
pub use dashboard::{Dashboard, FleetConfig};
pub use fleet_panel::{FleetHealth, FleetPanel, FleetStats};
pub use render::{Color, Render};
pub use room_panel::{AlarmSeverity, RoomPanel, RoomState, SensorReading};
pub use sparkline::Sparkline;
