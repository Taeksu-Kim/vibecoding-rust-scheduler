use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSession {
    /// 총 Pomodoro 수
    pub total_pomodoros: u32,
    
    /// 완료된 Pomodoro 수
    pub completed_pomodoros: u32,
    
    /// 현재 Pomodoro 시작 시간
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_start: Option<DateTime<Local>>,
    
    /// Pomodoro 길이 (분, 기본 25분)
    #[serde(default = "default_pomodoro_duration")]
    pub pomodoro_duration: u32,
    
    /// 짧은 휴식 (분, 기본 5분)
    #[serde(default = "default_short_break")]
    pub short_break: u32,
    
    /// 긴 휴식 (분, 기본 15분, 4 pomodoro 후)
    #[serde(default = "default_long_break")]
    pub long_break: u32,
}

fn default_pomodoro_duration() -> u32 { 25 }
fn default_short_break() -> u32 { 5 }
fn default_long_break() -> u32 { 15 }

impl Default for PomodoroSession {
    fn default() -> Self {
        Self {
            total_pomodoros: 0,
            completed_pomodoros: 0,
            current_start: None,
            pomodoro_duration: 25,
            short_break: 5,
            long_break: 15,
        }
    }
}

impl PomodoroSession {
    pub fn new(_estimated_minutes: i64) -> Self {
        // 기본값 25분으로 total 계산 (나중에 custom duration으로 업데이트됨)
        Self {
            total_pomodoros: 1, // Task.start()에서 재계산됨
            completed_pomodoros: 0,
            current_start: None,
            pomodoro_duration: 25,
            short_break: 5,
            long_break: 15,
        }
    }

    pub fn start_pomodoro(&mut self) {
        self.current_start = Some(Local::now());
    }

    pub fn complete_pomodoro(&mut self) {
        self.completed_pomodoros += 1;
        self.current_start = None;
    }

    pub fn elapsed_minutes(&self) -> Option<i64> {
        self.current_start.map(|start| {
            let now = Local::now();
            (now - start).num_minutes()
        })
    }

    pub fn remaining_minutes(&self) -> Option<i64> {
        self.elapsed_minutes().map(|elapsed| {
            (self.pomodoro_duration as i64 - elapsed).max(0)
        })
    }

    pub fn is_complete(&self) -> bool {
        self.completed_pomodoros >= self.total_pomodoros
    }

    pub fn next_break_duration(&self) -> u32 {
        if (self.completed_pomodoros + 1) % 4 == 0 {
            self.long_break
        } else {
            self.short_break
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pomodoro_creation() {
        let session = PomodoroSession::new(50);
        assert_eq!(session.total_pomodoros, 2); // 50min = 2 pomodoros
        assert_eq!(session.completed_pomodoros, 0);
    }

    #[test]
    fn test_pomodoro_completion() {
        let mut session = PomodoroSession::new(25);
        session.start_pomodoro();
        session.complete_pomodoro();
        
        assert_eq!(session.completed_pomodoros, 1);
        assert!(session.is_complete());
    }

    #[test]
    fn test_break_duration() {
        let mut session = PomodoroSession::new(100);
        
        assert_eq!(session.next_break_duration(), 5); // First break: short
        session.completed_pomodoros = 3;
        assert_eq!(session.next_break_duration(), 15); // 4th break: long
    }
}
