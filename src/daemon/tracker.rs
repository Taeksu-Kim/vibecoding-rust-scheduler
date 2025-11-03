use std::thread;
use std::time::Duration;
use chrono::Local;

use crate::models::{TaskStatus, DailyStats};
use crate::storage::{JsonStorage, Storage};

pub struct TimeTracker {
    storage: JsonStorage,
    running: bool,
}

impl TimeTracker {
    pub fn new(storage: JsonStorage) -> Self {
        Self {
            storage,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        log::info!("Time tracker started");

        while self.running {
            if let Err(e) = self.update() {
                log::error!("Tracker update error: {}", e);
            }

            thread::sleep(Duration::from_secs(60)); // 1분마다 업데이트
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
        log::info!("Time tracker stopped");
    }

    fn update(&self) -> anyhow::Result<()> {
        let schedule = match self.storage.load_today()? {
            Some(s) => s,
            None => return Ok(()), // 스케줄 없으면 스킵
        };

        // 현재 진행 중인 작업이 있는지 확인
        if let Some(current) = schedule.get_current_task() {
            log::debug!("Current task: {} - elapsed: {:?}min", 
                current.title, 
                current.elapsed_minutes()
            );

            // 시간 초과 경고
            if current.is_overdue() {
                log::warn!("Task '{}' is overdue!", current.title);
                // TODO: 알림 보내기
            }
        }

        // 통계 업데이트
        self.update_stats(&schedule)?;

        Ok(())
    }

    fn update_stats(&self, schedule: &crate::models::Schedule) -> anyhow::Result<()> {
        let mut stats = self.storage
            .load_stats(Local::now())?
            .unwrap_or_else(|| DailyStats::new(Local::now()));

        stats.completion_rate = schedule.completion_rate();
        stats.time_accuracy = schedule.time_accuracy();
        stats.total_tasks = schedule.tasks.len();
        stats.completed_tasks = schedule
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();

        // 집중 시간 계산 (완료된 작업의 실제 시간 합)
        stats.focus_time_minutes = schedule
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .filter_map(|t| t.actual_duration_minutes)
            .sum();

        self.storage.save_stats(&stats)?;

        Ok(())
    }
}
