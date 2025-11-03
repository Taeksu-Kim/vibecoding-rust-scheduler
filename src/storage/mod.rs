pub mod json_storage;

use chrono::{DateTime, Local};

use crate::models::{DailyStats, Schedule, StreakInfo};

pub trait Storage {
    fn save_schedule(&self, schedule: &Schedule) -> anyhow::Result<()>;
    fn load_schedule(&self, date: DateTime<Local>) -> anyhow::Result<Option<Schedule>>;
    fn load_today(&self) -> anyhow::Result<Option<Schedule>>;
    fn save_stats(&self, stats: &DailyStats) -> anyhow::Result<()>;
    fn load_stats(&self, date: DateTime<Local>) -> anyhow::Result<Option<DailyStats>>;
    fn save_streak(&self, streak: &StreakInfo) -> anyhow::Result<()>;
    fn load_streak(&self) -> anyhow::Result<StreakInfo>;
}

pub use json_storage::JsonStorage;
