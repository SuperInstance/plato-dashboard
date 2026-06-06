use plato_dashboard::{Dashboard, FleetConfig, RoomState, SensorReading, AlarmSeverity};

fn main() {
    // Demo with 3 mock rooms
    let rooms = vec![
        RoomState {
            name: "Engine Room".to_string(),
            tick_rate_ms: 1000,
            last_tick_age_ms: 150,
            sensors: vec![
                SensorReading { name: "Temp".to_string(), value: 87.3, unit: "°C".to_string(), alarm: AlarmSeverity::Warning },
                SensorReading { name: "Humidity".to_string(), value: 62.1, unit: "%".to_string(), alarm: AlarmSeverity::Clear },
                SensorReading { name: "Pressure".to_string(), value: 1013.2, unit: "hPa".to_string(), alarm: AlarmSeverity::Clear },
            ],
            history: vec![
                vec![80.0, 82.0, 84.0, 85.0, 86.0, 87.0, 87.3],
                vec![58.0, 59.0, 60.0, 61.0, 62.0, 62.0, 62.1],
                vec![1013.0, 1013.1, 1013.2, 1013.2, 1013.2, 1013.2, 1013.2],
            ],
            online: true,
        },
        RoomState {
            name: "Bridge".to_string(),
            tick_rate_ms: 2000,
            last_tick_age_ms: 300,
            sensors: vec![
                SensorReading { name: "Temp".to_string(), value: 22.1, unit: "°C".to_string(), alarm: AlarmSeverity::Clear },
                SensorReading { name: "CO2".to_string(), value: 1200.0, unit: "ppm".to_string(), alarm: AlarmSeverity::Critical },
            ],
            history: vec![
                vec![21.0, 21.5, 22.0, 22.0, 22.1],
                vec![400.0, 600.0, 800.0, 1000.0, 1200.0],
            ],
            online: true,
        },
        RoomState {
            name: "Cargo Hold".to_string(),
            tick_rate_ms: 5000,
            last_tick_age_ms: 30000,
            sensors: vec![
                SensorReading { name: "Temp".to_string(), value: 15.2, unit: "°C".to_string(), alarm: AlarmSeverity::Clear },
            ],
            history: vec![vec![15.0, 15.1, 15.2]],
            online: false,
        },
    ];

    let config = FleetConfig::new(rooms);
    let mut dashboard = Dashboard::new(config);

    // Add some alarms
    use plato_dashboard::alarm_panel::AlarmEvent;
    dashboard.alarm_panel_mut().add_alarm(AlarmEvent {
        id: 1,
        room_name: "Engine Room".to_string(),
        sensor_name: "Temp".to_string(),
        severity: AlarmSeverity::Warning,
        message: "Temperature elevated".to_string(),
        timestamp_secs: 1000,
        acknowledged: false,
    });
    dashboard.alarm_panel_mut().add_alarm(AlarmEvent {
        id: 2,
        room_name: "Bridge".to_string(),
        sensor_name: "CO2".to_string(),
        severity: AlarmSeverity::Critical,
        message: "CO2 exceeds safe limit".to_string(),
        timestamp_secs: 1005,
        acknowledged: false,
    });

    dashboard.set_running(true);
    println!("{}", dashboard.render_full());
}
