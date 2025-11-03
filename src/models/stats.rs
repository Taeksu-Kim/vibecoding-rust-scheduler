use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// 하루 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    /// 날짜
    pub date: DateTime<Local>,

    /// 완료율 (%)
    pub completion_rate: f64,

    /// 시간 정확도 (%)
    pub time_accuracy: Option<f64>,

    /// 총 작업 개수
    pub total_tasks: usize,

    /// 완료된 작업 개수
    pub completed_tasks: usize,

    /// 집중 시간 (분)
    pub focus_time_minutes: i64,

    /// 휴식 시간 (분)
    pub break_time_minutes: i64,
}

impl DailyStats {
    pub fn new(date: DateTime<Local>) -> Self {
        Self {
            date,
            completion_rate: 0.0,
            time_accuracy: None,
            total_tasks: 0,
            completed_tasks: 0,
            focus_time_minutes: 0,
            break_time_minutes: 0,
        }
    }
}

/// Streak 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakInfo {
    /// 현재 연속 일수
    pub current_streak: u32,

    /// 최고 기록
    pub best_streak: u32,

    /// 마지막 업데이트 날짜
    pub last_update: DateTime<Local>,
}

impl StreakInfo {
    pub fn new() -> Self {
        Self {
            current_streak: 0,
            best_streak: 0,
            last_update: Local::now(),
        }
    }

    /// Streak 업데이트 (완료율 기준)
    pub fn update(&mut self, completion_rate: f64) {
        let now = Local::now();

        // 70% 이상 완료 시 성공으로 간주
        if completion_rate >= 70.0 {
            self.current_streak += 1;

            if self.current_streak > self.best_streak {
                self.best_streak = self.current_streak;
            }
        } else {
            self.current_streak = 0;
        }

        self.last_update = now;
    }

    /// Streak 초기화 (깨짐)
    pub fn reset(&mut self) {
        self.current_streak = 0;
        self.last_update = Local::now();
    }
}

impl Default for StreakInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_stats_creation() {
        let stats = DailyStats::new(Local::now());
        assert_eq!(stats.completion_rate, 0.0);
        assert_eq!(stats.total_tasks, 0);
    }

    #[test]
    fn test_streak_update() {
        let mut streak = StreakInfo::new();

        streak.update(80.0);
        assert_eq!(streak.current_streak, 1);

        streak.update(90.0);
        assert_eq!(streak.current_streak, 2);
        assert_eq!(streak.best_streak, 2);

        streak.update(50.0); // 실패
        assert_eq!(streak.current_streak, 0);
        assert_eq!(streak.best_streak, 2); // 최고 기록은 유지
    }
}
