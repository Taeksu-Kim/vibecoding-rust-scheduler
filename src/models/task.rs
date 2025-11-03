use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 대기 중
    Pending,
    /// 진행 중
    InProgress,
    /// 완료
    Completed,
    /// 일시정지
    Paused,
    /// 건너뜀
    Skipped,
}

/// 하나의 작업 (Task)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 작업 ID (UUID)
    pub id: String,

    /// 작업 제목
    pub title: String,

    /// 시작 시간
    pub start_time: DateTime<Local>,

    /// 종료 시간
    pub end_time: DateTime<Local>,

    /// 예상 소요 시간 (분)
    pub estimated_duration_minutes: i64,

    /// 실제 소요 시간 (분, Optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_duration_minutes: Option<i64>,

    /// 현재 상태
    pub status: TaskStatus,

    /// 태그
    #[serde(default)]
    pub tags: Vec<String>,

    /// 메모
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// 실제 시작된 시간 (InProgress 시)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_start_time: Option<DateTime<Local>>,

    /// 실제 완료된 시간 (Completed 시)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_end_time: Option<DateTime<Local>>,

    /// 사용자 정의 Pomodoro 시간 (분, 기본값 25분)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_pomodoro_duration: Option<u32>,

    /// Pomodoro session (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pomodoro: Option<super::pomodoro::PomodoroSession>,
}

impl Task {
    /// 새 Task 생성
    pub fn new(
        title: String,
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
    ) -> Self {
        let duration = (end_time - start_time).num_minutes();

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            start_time,
            end_time,
            estimated_duration_minutes: duration,
            actual_duration_minutes: None,
            status: TaskStatus::Pending,
            tags: Vec::new(),
            notes: None,
            actual_start_time: None,
            actual_end_time: None,
            custom_pomodoro_duration: None,
            pomodoro: None,
        }
    }

    /// 작업 시작
    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.actual_start_time = Some(Local::now());

        // Pomodoro 세션 시작
        if self.pomodoro.is_none() {
            let pomodoro_duration = self.custom_pomodoro_duration.unwrap_or(25);
            let mut session = super::pomodoro::PomodoroSession::new(self.estimated_duration_minutes);
            session.pomodoro_duration = pomodoro_duration;
            // total_pomodoros를 custom duration 기준으로 재계산
            session.total_pomodoros = ((self.estimated_duration_minutes as f64 / pomodoro_duration as f64).ceil() as u32).max(1);
            session.start_pomodoro();
            self.pomodoro = Some(session);
        } else if let Some(ref mut session) = self.pomodoro {
            session.start_pomodoro();
        }
    }

    /// 작업 일시정지
    pub fn pause(&mut self) {
        if self.status == TaskStatus::InProgress {
            self.status = TaskStatus::Paused;

            // Pomodoro도 일시정지 (current_start를 None으로)
            if let Some(ref mut session) = self.pomodoro {
                session.current_start = None;
            }
        }
    }

    /// 작업 재개
    pub fn resume(&mut self) {
        if self.status == TaskStatus::Paused {
            self.status = TaskStatus::InProgress;

            // Pomodoro도 재개
            if let Some(ref mut session) = self.pomodoro {
                session.start_pomodoro();
            }
        }
    }

    /// 작업 완료
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.actual_end_time = Some(Local::now());

        // 실제 소요 시간 계산
        if let Some(start) = self.actual_start_time {
            let end = self.actual_end_time.unwrap();
            self.actual_duration_minutes = Some((end - start).num_minutes());
        }
    }

    /// 작업 건너뛰기
    pub fn skip(&mut self) {
        self.status = TaskStatus::Skipped;
    }

    /// 경과 시간 (분)
    pub fn elapsed_minutes(&self) -> Option<i64> {
        if let Some(start) = self.actual_start_time {
            let now = Local::now();
            Some((now - start).num_minutes())
        } else {
            None
        }
    }

    /// 예상 시간 초과 여부
    pub fn is_overdue(&self) -> bool {
        if let Some(elapsed) = self.elapsed_minutes() {
            elapsed > self.estimated_duration_minutes
        } else {
            false
        }
    }

    /// 현재 진행 중인지
    pub fn is_current(&self) -> bool {
        self.status == TaskStatus::InProgress
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_task_creation() {
        let start = Local::now();
        let end = start + Duration::hours(2);
        let task = Task::new("Test Task".to_string(), start, end);

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.estimated_duration_minutes, 120);
        assert!(task.actual_duration_minutes.is_none());
    }

    #[test]
    fn test_task_start_complete() {
        let start = Local::now();
        let end = start + Duration::hours(1);
        let mut task = Task::new("Test".to_string(), start, end);

        task.start();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.actual_start_time.is_some());

        task.complete();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.actual_end_time.is_some());
        assert!(task.actual_duration_minutes.is_some());
    }

    #[test]
    fn test_task_pause_resume() {
        let start = Local::now();
        let end = start + Duration::hours(1);
        let mut task = Task::new("Test".to_string(), start, end);

        task.start();
        task.pause();
        assert_eq!(task.status, TaskStatus::Paused);

        task.resume();
        assert_eq!(task.status, TaskStatus::InProgress);
    }
}
