use plato_dashboard::*;

fn main() {
    let config = DashboardConfig::new("F/V AURORA");

    let rooms = vec![
        RoomSummary::new("Engine", RoomType::Engine, RoomStatus::Warning, 89.1, 0.72, 5),
        RoomSummary::new("Bridge", RoomType::Bridge, RoomStatus::Ok, 98.4, 0.95, 2),
        RoomSummary::new("Galley", RoomType::Galley, RoomStatus::Ok, 97.2, 0.91, 3),
        RoomSummary::new("Hold", RoomType::Hold, RoomStatus::Critical, 45.3, 0.34, 12),
    ];

    let alerts = vec![
        AlertSummary::new("Engine", "overheat", "Engine temp above threshold", Severity::Warning, 1000),
        AlertSummary::new("Hold", "pressure", "Hold pressure critical", Severity::Critical, 1005),
        AlertSummary::new("Bridge", "info", "Navigation update available", Severity::Info, 1010),
    ];

    let dashboard = Dashboard::new(&config, rooms, alerts, 67423);

    println!("{}", dashboard.render_text());
}
