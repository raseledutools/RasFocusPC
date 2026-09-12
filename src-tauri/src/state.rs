use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleEntry {
    pub enabled: bool,
    pub start_time: String, // "HH:MM"
    pub end_time: String,   // "HH:MM"
    pub days: Vec<String>,  // ["Mon","Tue",...]
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub custom_sites: Vec<String>,
    pub blocking_active: bool,
    pub active_presets: Vec<String>,
    pub schedule: Option<ScheduleEntry>,
}
