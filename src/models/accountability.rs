use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use super::{Task, TaskStatus};

/// 개별 작업의 시간 성과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAccountability {
    /// 지켜진 시간 (분) - 계획대로 완료한 시간
    pub earned_time: i64,
    /// 낭비한 시간 (분) - 건너뛰거나 미완료한 시간
    pub wasted_time: i64,
    /// 보너스 시간 (분) - 예상보다 빨리 완료
    pub bonus_time: i64,
    /// 페널티 시간 (분) - 예상보다 늦게 완료
    pub penalty_time: i64,
}

impl TimeAccountability {
    /// Task로부터 시간 성과 계산
    pub fn from_task(task: &Task) -> Self {
        let estimated = task.estimated_duration_minutes;

        match task.status {
            TaskStatus::Completed => {
                if let Some(actual) = task.actual_duration_minutes {
                    if actual <= estimated {
                        // 시간 내 완료 또는 빨리 완료
                        let bonus = estimated - actual;
                        Self {
                            earned_time: estimated,
                            wasted_time: 0,
                            bonus_time: bonus,
                            penalty_time: 0,
                        }
                    } else {
                        // 예상보다 늦게 완료
                        let penalty = actual - estimated;
                        Self {
                            earned_time: estimated.saturating_sub(penalty),
                            wasted_time: 0,
                            bonus_time: 0,
                            penalty_time: penalty,
                        }
                    }
                } else {
                    // actual_duration이 없는 경우 (정상 완료로 간주)
                    Self {
                        earned_time: estimated,
                        wasted_time: 0,
                        bonus_time: 0,
                        penalty_time: 0,
                    }
                }
            }
            TaskStatus::Skipped => {
                // 건너뜀 - 전체 시간 낭비
                Self {
                    earned_time: 0,
                    wasted_time: estimated,
                    bonus_time: 0,
                    penalty_time: 0,
                }
            }
            TaskStatus::Pending | TaskStatus::InProgress | TaskStatus::Paused => {
                // 미완료 - 낭비로 간주하지 않음 (아직 진행 가능)
                Self {
                    earned_time: 0,
                    wasted_time: 0,
                    bonus_time: 0,
                    penalty_time: 0,
                }
            }
        }
    }

    /// 순수 획득 시간 (earned + bonus - penalty)
    pub fn net_earned(&self) -> i64 {
        self.earned_time + self.bonus_time - self.penalty_time
    }

    /// 성과 메시지 생성
    pub fn feedback_message(&self) -> Option<String> {
        if self.earned_time > 0 && self.bonus_time > 0 {
            Some(format!("+{}m earned, +{}m bonus! 🎉", self.earned_time, self.bonus_time))
        } else if self.earned_time > 0 && self.penalty_time > 0 {
            Some(format!("+{}m earned, -{}m penalty ⚠", self.earned_time, self.penalty_time))
        } else if self.earned_time > 0 {
            Some(format!("+{}m earned ✓", self.earned_time))
        } else if self.wasted_time > 0 {
            Some(format!("-{}m wasted ✗", self.wasted_time))
        } else {
            None
        }
    }
}

/// 일일 시간 책임성 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyAccountability {
    pub date: DateTime<Local>,
    /// 총 계획된 시간 (분)
    pub total_planned: i64,
    /// 총 지켜진 시간 (분)
    pub total_earned: i64,
    /// 총 낭비한 시간 (분)
    pub total_wasted: i64,
    /// 총 보너스 시간 (분)
    pub total_bonus: i64,
    /// 총 페널티 시간 (분)
    pub total_penalty: i64,
}

impl DailyAccountability {
    pub fn new(date: DateTime<Local>) -> Self {
        Self {
            date,
            total_planned: 0,
            total_earned: 0,
            total_wasted: 0,
            total_bonus: 0,
            total_penalty: 0,
        }
    }

    /// Task 목록으로부터 일일 통계 계산
    pub fn from_tasks(date: DateTime<Local>, tasks: &[Task]) -> Self {
        let mut accountability = Self::new(date);

        for task in tasks {
            accountability.total_planned += task.estimated_duration_minutes;

            let perf = TimeAccountability::from_task(task);
            accountability.total_earned += perf.earned_time;
            accountability.total_wasted += perf.wasted_time;
            accountability.total_bonus += perf.bonus_time;
            accountability.total_penalty += perf.penalty_time;
        }

        accountability
    }

    /// 시간 효율 점수 (0-100%)
    pub fn efficiency_score(&self) -> f64 {
        if self.total_planned == 0 {
            return 0.0;
        }

        let net_earned = self.total_earned + self.total_bonus - self.total_penalty;
        (net_earned as f64 / self.total_planned as f64) * 100.0
    }

    /// 순수 획득 시간
    pub fn net_earned(&self) -> i64 {
        self.total_earned + self.total_bonus - self.total_penalty
    }

    /// 효율 등급 (A+, A, B, C, D, F)
    pub fn grade(&self) -> &'static str {
        let score = self.efficiency_score();
        if score >= 95.0 {
            "A+"
        } else if score >= 90.0 {
            "A"
        } else if score >= 80.0 {
            "B"
        } else if score >= 70.0 {
            "C"
        } else if score >= 60.0 {
            "D"
        } else {
            "F"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_on_time_completion() {
        let mut task = Task::new(
            "Test".to_string(),
            Local.with_ymd_and_hms(2025, 11, 1, 9, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 11, 1, 10, 0, 0).unwrap(),
            60,
            vec![],
            None,
        );
        task.status = TaskStatus::Completed;
        task.actual_duration_minutes = Some(60);

        let perf = TimeAccountability::from_task(&task);
        assert_eq!(perf.earned_time, 60);
        assert_eq!(perf.bonus_time, 0);
        assert_eq!(perf.penalty_time, 0);
        assert_eq!(perf.wasted_time, 0);
    }

    #[test]
    fn test_early_completion() {
        let mut task = Task::new(
            "Test".to_string(),
            Local.with_ymd_and_hms(2025, 11, 1, 9, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 11, 1, 10, 0, 0).unwrap(),
            60,
            vec![],
            None,
        );
        task.status = TaskStatus::Completed;
        task.actual_duration_minutes = Some(45);

        let perf = TimeAccountability::from_task(&task);
        assert_eq!(perf.earned_time, 60);
        assert_eq!(perf.bonus_time, 15);
        assert_eq!(perf.penalty_time, 0);
    }

    #[test]
    fn test_late_completion() {
        let mut task = Task::new(
            "Test".to_string(),
            Local.with_ymd_and_hms(2025, 11, 1, 9, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 11, 1, 10, 0, 0).unwrap(),
            60,
            vec![],
            None,
        );
        task.status = TaskStatus::Completed;
        task.actual_duration_minutes = Some(75);

        let perf = TimeAccountability::from_task(&task);
        assert_eq!(perf.earned_time, 45);
        assert_eq!(perf.penalty_time, 15);
    }

    #[test]
    fn test_skipped_task() {
        let mut task = Task::new(
            "Test".to_string(),
            Local.with_ymd_and_hms(2025, 11, 1, 9, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 11, 1, 10, 0, 0).unwrap(),
            60,
            vec![],
            None,
        );
        task.status = TaskStatus::Skipped;

        let perf = TimeAccountability::from_task(&task);
        assert_eq!(perf.wasted_time, 60);
        assert_eq!(perf.earned_time, 0);
    }

    #[test]
    fn test_efficiency_score() {
        let date = Local.with_ymd_and_hms(2025, 11, 1, 0, 0, 0).unwrap();
        let mut daily = DailyAccountability::new(date);
        daily.total_planned = 240; // 4 hours
        daily.total_earned = 200;
        daily.total_bonus = 30;
        daily.total_penalty = 10;

        // (200 + 30 - 10) / 240 = 220/240 = 91.67%
        assert!((daily.efficiency_score() - 91.67).abs() < 0.1);
        assert_eq!(daily.grade(), "A");
    }
}
