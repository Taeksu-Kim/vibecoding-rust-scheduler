use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use directories::ProjectDirs;

use crate::models::{DailyStats, Schedule, StreakInfo};

use super::Storage;

/// JSON 파일 기반 Storage
pub struct JsonStorage {
    data_dir: PathBuf,
}

impl JsonStorage {
    /// 새 JsonStorage 생성
    pub fn new() -> anyhow::Result<Self> {
        let project_dirs = ProjectDirs::from("com", "scheduler", "scheduler")
            .ok_or_else(|| anyhow::anyhow!("Failed to determine project directory"))?;

        let data_dir = project_dirs.data_dir().to_path_buf();

        // 디렉토리 생성
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("history"))?;

        Ok(Self { data_dir })
    }

    /// 커스텀 경로로 생성 (테스트용)
    pub fn with_path(path: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(&path)?;
        fs::create_dir_all(path.join("history"))?;
        Ok(Self { data_dir: path })
    }

    /// 날짜를 파일명으로 변환
    fn date_to_filename(&self, date: DateTime<Local>) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    /// 스케줄 파일 경로
    fn schedule_path(&self, date: DateTime<Local>) -> PathBuf {
        let filename = format!("{}.json", self.date_to_filename(date));
        self.data_dir.join("history").join(filename)
    }

    /// 현재 스케줄 파일 경로
    fn current_schedule_path(&self) -> PathBuf {
        self.data_dir.join("current.json")
    }

    /// 통계 파일 경로
    fn stats_path(&self, date: DateTime<Local>) -> PathBuf {
        let filename = format!("{}_stats.json", self.date_to_filename(date));
        self.data_dir.join("history").join(filename)
    }

    /// Streak 파일 경로
    fn streak_path(&self) -> PathBuf {
        self.data_dir.join("streak.json")
    }
}

impl Storage for JsonStorage {
    fn save_schedule(&self, schedule: &Schedule) -> anyhow::Result<()> {
        // history에 저장
        let history_path = self.schedule_path(schedule.date);
        let json = serde_json::to_string_pretty(schedule)?;
        fs::write(history_path, json)?;

        // current.json 업데이트 (오늘 날짜인 경우)
        let today = Local::now().date_naive();
        let schedule_date = schedule.date.date_naive();

        if schedule_date == today {
            let current_path = self.current_schedule_path();
            let json = serde_json::to_string_pretty(schedule)?;
            fs::write(current_path, json)?;
        }

        Ok(())
    }

    fn load_schedule(&self, date: DateTime<Local>) -> anyhow::Result<Option<Schedule>> {
        let path = self.schedule_path(date);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let schedule: Schedule = serde_json::from_str(&content)?;
        Ok(Some(schedule))
    }

    fn load_today(&self) -> anyhow::Result<Option<Schedule>> {
        let path = self.current_schedule_path();

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let schedule: Schedule = serde_json::from_str(&content)?;

        // 날짜 검증 (오늘이 아니면 None)
        let today = Local::now().date_naive();
        if schedule.date.date_naive() != today {
            return Ok(None);
        }

        Ok(Some(schedule))
    }

    fn save_stats(&self, stats: &DailyStats) -> anyhow::Result<()> {
        let path = self.stats_path(stats.date);
        let json = serde_json::to_string_pretty(stats)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn load_stats(&self, date: DateTime<Local>) -> anyhow::Result<Option<DailyStats>> {
        let path = self.stats_path(date);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let stats: DailyStats = serde_json::from_str(&content)?;
        Ok(Some(stats))
    }

    fn save_streak(&self, streak: &StreakInfo) -> anyhow::Result<()> {
        let path = self.streak_path();
        let json = serde_json::to_string_pretty(streak)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn load_streak(&self) -> anyhow::Result<StreakInfo> {
        let path = self.streak_path();

        if !path.exists() {
            return Ok(StreakInfo::new());
        }

        let content = fs::read_to_string(path)?;
        let streak: StreakInfo = serde_json::from_str(&content)?;
        Ok(streak)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;
    use chrono::Duration;

    #[test]
    fn test_json_storage_schedule() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = JsonStorage::with_path(temp_dir.path().to_path_buf()).unwrap();

        let mut schedule = Schedule::today();
        let start = Local::now();
        let task = Task::new("Test".to_string(), start, start + Duration::hours(1));
        schedule.add_task(task).unwrap();

        // 저장
        storage.save_schedule(&schedule).unwrap();

        // 불러오기
        let loaded = storage.load_today().unwrap();
        assert!(loaded.is_some());

        let loaded_schedule = loaded.unwrap();
        assert_eq!(loaded_schedule.tasks.len(), 1);
        assert_eq!(loaded_schedule.tasks[0].title, "Test");
    }

    #[test]
    fn test_json_storage_streak() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = JsonStorage::with_path(temp_dir.path().to_path_buf()).unwrap();

        let mut streak = StreakInfo::new();
        streak.update(80.0);

        // 저장
        storage.save_streak(&streak).unwrap();

        // 불러오기
        let loaded = storage.load_streak().unwrap();
        assert_eq!(loaded.current_streak, 1);
    }

    #[test]
    fn test_json_storage_stats() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = JsonStorage::with_path(temp_dir.path().to_path_buf()).unwrap();

        let mut stats = DailyStats::new(Local::now());
        stats.completion_rate = 75.0;
        stats.total_tasks = 4;
        stats.completed_tasks = 3;

        // 저장
        storage.save_stats(&stats).unwrap();

        // 불러오기
        let loaded = storage.load_stats(Local::now()).unwrap();
        assert!(loaded.is_some());

        let loaded_stats = loaded.unwrap();
        assert_eq!(loaded_stats.completion_rate, 75.0);
        assert_eq!(loaded_stats.total_tasks, 4);
    }
}
