use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use super::task::{Task, TaskStatus};

/// 스케줄 변경 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// 작업 생성
    TaskCreated,
    /// 작업 수정 (시간 변경, 제목 변경 등)
    TaskUpdated,
    /// 작업 삭제
    TaskDeleted,
    /// 작업 드래그 앤 드롭으로 이동
    TaskMoved,
    /// 스케줄 미루기
    ScheduleShifted,
}

/// 스케줄 변경 이력
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleChange {
    /// 변경 시각
    pub timestamp: DateTime<Local>,
    /// 변경 타입
    pub change_type: ChangeType,
    /// 작업 제목 (해당하는 경우)
    pub task_title: Option<String>,
    /// 변경 전 시간 (HH:MM)
    pub old_time: Option<String>,
    /// 변경 후 시간 (HH:MM)
    pub new_time: Option<String>,
    /// 영향받은 작업 수 (shift의 경우)
    pub affected_tasks_count: Option<usize>,
    /// 추가 설명
    pub description: String,
}

impl ScheduleChange {
    /// 작업 이동 변경 생성
    pub fn task_moved(task_title: String, old_time: String, new_time: String) -> Self {
        Self {
            timestamp: Local::now(),
            change_type: ChangeType::TaskMoved,
            task_title: Some(task_title.clone()),
            old_time: Some(old_time.clone()),
            new_time: Some(new_time.clone()),
            affected_tasks_count: None,
            description: format!("\"{}\"를 {}에서 {}로 이동", task_title, old_time, new_time),
        }
    }

    /// 스케줄 미루기/당기기 변경 생성
    pub fn schedule_shifted(from_task: String, minutes: i64, affected_count: usize) -> Self {
        let direction = if minutes > 0 { "미루기" } else { "당기기" };
        let abs_minutes = minutes.abs();
        Self {
            timestamp: Local::now(),
            change_type: ChangeType::ScheduleShifted,
            task_title: Some(from_task.clone()),
            old_time: None,
            new_time: None,
            affected_tasks_count: Some(affected_count),
            description: format!(
                "\"{}\" 이후 {}개 작업 {}분 {} (영향: {}개 작업)",
                from_task, affected_count, abs_minutes, direction, affected_count
            ),
        }
    }

    /// 작업 수정 변경 생성
    pub fn task_updated(task_title: String, old_time: String, new_time: String) -> Self {
        Self {
            timestamp: Local::now(),
            change_type: ChangeType::TaskUpdated,
            task_title: Some(task_title.clone()),
            old_time: Some(old_time.clone()),
            new_time: Some(new_time.clone()),
            affected_tasks_count: None,
            description: format!("\"{}\" 시간 변경: {} → {}", task_title, old_time, new_time),
        }
    }
}

/// 하루 스케줄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// 날짜
    pub date: DateTime<Local>,

    /// 작업 목록
    pub tasks: Vec<Task>,

    /// 변경 이력
    #[serde(default)]
    pub changes: Vec<ScheduleChange>,

    /// 완료율 (계산된 값)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_rate: Option<f64>,

    /// 효율 점수 (계산된 값)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub efficiency_score: Option<f64>,

    /// 획득한 시간 (계산된 값)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_earned: Option<i64>,

    /// 낭비한 시간 (계산된 값)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_wasted: Option<i64>,

    /// 보너스 시간 (계산된 값)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bonus: Option<i64>,

    /// 페널티 시간 (계산된 값)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_penalty: Option<i64>,
}

impl Schedule {
    /// 새 스케줄 생성
    pub fn new(date: DateTime<Local>) -> Self {
        Self {
            date,
            tasks: Vec::new(),
            changes: Vec::new(),
            completion_rate: None,
            efficiency_score: None,
            total_earned: None,
            total_wasted: None,
            total_bonus: None,
            total_penalty: None,
        }
    }

    /// 변경 이력 추가
    pub fn add_change(&mut self, change: ScheduleChange) {
        self.changes.push(change);
    }

    /// 통계 계산 및 업데이트
    pub fn calculate_stats(&mut self) {
        self.completion_rate = Some(self.completion_rate());
        self.efficiency_score = Some(self.efficiency_score());
        self.total_earned = Some(self.total_earned());
        // total_wasted는 현재 시간 기준이므로 저장하지 않음 (매번 계산)
        self.total_bonus = Some(self.total_bonus());
        self.total_penalty = Some(self.total_penalty());
    }

    /// 오늘 스케줄 생성
    pub fn today() -> Self {
        Self::new(Local::now())
    }

    /// 작업 추가
    pub fn add_task(&mut self, task: Task) -> Result<(), String> {
        // 시간 충돌 검사
        for existing_task in &self.tasks {
            if self.has_time_conflict(&task, existing_task) {
                return Err(format!(
                    "Time conflict with task: {}",
                    existing_task.title
                ));
            }
        }

        self.tasks.push(task);
        Ok(())
    }

    /// 작업 제거
    pub fn remove_task(&mut self, task_id: &str) -> Option<Task> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == task_id) {
            Some(self.tasks.remove(pos))
        } else {
            None
        }
    }

    /// 작업 찾기
    pub fn find_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    /// 작업 찾기 (mutable)
    pub fn find_task_mut(&mut self, task_id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id)
    }

    /// 현재 진행 중인 작업
    pub fn get_current_task(&self) -> Option<&Task> {
        self.tasks.iter().find(|t| t.is_current())
    }

    /// 다음 작업 (Pending 상태 중 가장 빠른 시작 시간)
    pub fn get_next_task(&self) -> Option<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .min_by_key(|t| t.start_time)
    }

    /// 완료율 계산 (%)
    pub fn completion_rate(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }

        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();

        (completed as f64 / self.tasks.len() as f64) * 100.0
    }

    /// 시간 정확도 계산 (%)
    pub fn time_accuracy(&self) -> Option<f64> {
        let completed_tasks: Vec<_> = self
            .tasks
            .iter()
            .filter(|t| {
                t.status == TaskStatus::Completed && t.actual_duration_minutes.is_some()
            })
            .collect();

        if completed_tasks.is_empty() {
            return None;
        }

        let total_accuracy: f64 = completed_tasks
            .iter()
            .map(|t| {
                let estimated = t.estimated_duration_minutes as f64;
                let actual = t.actual_duration_minutes.unwrap() as f64;
                let diff = (estimated - actual).abs();
                ((estimated - diff) / estimated * 100.0).max(0.0)
            })
            .sum();

        Some(total_accuracy / completed_tasks.len() as f64)
    }

    /// 지켜진 시간 계산 (Earned Time) - 분 단위
    pub fn total_earned(&self) -> i64 {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| {
                let estimated = t.estimated_duration_minutes;
                let actual = t.actual_duration_minutes.unwrap_or(estimated);

                if actual <= estimated {
                    // 예상 시간 내 완료 또는 빨리 완료 -> 예상 시간만큼 획득
                    estimated
                } else {
                    // 초과 완료 -> 예상 시간에서 초과분만큼 차감
                    (estimated - (actual - estimated)).max(0)
                }
            })
            .sum()
    }

    /// 낭비한 시간 계산 (Wasted Time) - 분 단위
    /// 현재 시간 기준으로 이미 지나간 task만 계산
    pub fn total_wasted(&self) -> i64 {
        let now = Local::now();

        self.tasks
            .iter()
            .filter(|t| {
                // 완료되지 않았고, 종료 시간이 현재보다 과거인 task만
                t.status != TaskStatus::Completed && t.end_time < now
            })
            .map(|t| {
                if t.status == TaskStatus::Skipped {
                    // 건너뛴 작업 -> 예상 시간 전체가 낭비
                    t.estimated_duration_minutes
                } else {
                    // InProgress, Paused, Pending이지만 이미 종료 시간이 지난 경우
                    // 예상 시간 전체가 낭비 (시간을 지키지 못함)
                    t.estimated_duration_minutes
                }
            })
            .sum()
    }

    /// 보너스 시간 계산 - 예상보다 빨리 완료한 경우
    pub fn total_bonus(&self) -> i64 {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .filter_map(|t| {
                let estimated = t.estimated_duration_minutes;
                let actual = t.actual_duration_minutes?;

                if actual < estimated {
                    Some(estimated - actual)
                } else {
                    None
                }
            })
            .sum()
    }

    /// 페널티 시간 계산 - 예상보다 늦게 완료한 경우
    pub fn total_penalty(&self) -> i64 {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .filter_map(|t| {
                let estimated = t.estimated_duration_minutes;
                let actual = t.actual_duration_minutes?;

                if actual > estimated {
                    Some(actual - estimated)
                } else {
                    None
                }
            })
            .sum()
    }

    /// 시간 효율 점수 계산 (%)
    pub fn efficiency_score(&self) -> f64 {
        let total_planned: i64 = self.tasks.iter().map(|t| t.estimated_duration_minutes).sum();

        if total_planned == 0 {
            return 0.0;
        }

        let earned = self.total_earned() as f64;
        let planned = total_planned as f64;

        (earned / planned * 100.0).min(100.0)
    }

    /// 시간 충돌 검사
    fn has_time_conflict(&self, task1: &Task, task2: &Task) -> bool {
        // 시작 시간이 겹치는지 확인
        (task1.start_time >= task2.start_time && task1.start_time < task2.end_time)
            || (task1.end_time > task2.start_time && task1.end_time <= task2.end_time)
            || (task1.start_time <= task2.start_time && task1.end_time >= task2.end_time)
    }

    /// 시간순 정렬
    pub fn sort_by_time(&mut self) {
        self.tasks.sort_by_key(|t| t.start_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_schedule_creation() {
        let schedule = Schedule::today();
        assert_eq!(schedule.tasks.len(), 0);
    }

    #[test]
    fn test_add_task() {
        let mut schedule = Schedule::today();
        let start = Local::now();
        let end = start + Duration::hours(2);
        let task = Task::new("Test".to_string(), start, end);

        assert!(schedule.add_task(task).is_ok());
        assert_eq!(schedule.tasks.len(), 1);
    }

    #[test]
    fn test_time_conflict() {
        let mut schedule = Schedule::today();
        let start = Local::now();

        let task1 = Task::new("Task 1".to_string(), start, start + Duration::hours(2));
        let task2 = Task::new(
            "Task 2".to_string(),
            start + Duration::hours(1),
            start + Duration::hours(3),
        );

        assert!(schedule.add_task(task1).is_ok());
        assert!(schedule.add_task(task2).is_err()); // 시간 충돌
    }

    #[test]
    fn test_completion_rate() {
        let mut schedule = Schedule::today();
        let start = Local::now();

        let mut task1 = Task::new("Task 1".to_string(), start, start + Duration::hours(1));
        let task2 = Task::new(
            "Task 2".to_string(),
            start + Duration::hours(2),
            start + Duration::hours(3),
        );

        task1.complete();

        schedule.add_task(task1).unwrap();
        schedule.add_task(task2).unwrap();

        assert_eq!(schedule.completion_rate(), 50.0);
    }
}
