use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::models::{Schedule, Task, TaskStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleContext {
    pub current_time: String,
    pub current_task: Option<TaskInfo>,
    pub next_task: Option<TaskInfo>,
    pub today_schedule: ScheduleSummary,
    pub git_info: Option<GitInfo>,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskInfo {
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
    pub elapsed_minutes: Option<i64>,
    pub estimated_duration: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleSummary {
    pub date: String,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub in_progress_tasks: usize,
    pub pending_tasks: usize,
    pub completion_rate: f64,
    pub total_estimated_minutes: i64,
    pub total_actual_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitInfo {
    pub branch: String,
    pub status: String,
    pub recent_commits: Vec<String>,
}

impl ScheduleContext {
    pub fn collect(schedule: &Schedule) -> Self {
        let current_time = Local::now().format("%Y-%m-%d %H:%M").to_string();
        let working_directory = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        let current_task = schedule.get_current_task().map(TaskInfo::from_task);
        let next_task = schedule.get_next_task().map(TaskInfo::from_task);

        let today_schedule = ScheduleSummary::from_schedule(schedule);
        let git_info = Self::collect_git_info();

        Self {
            current_time,
            current_task,
            next_task,
            today_schedule,
            git_info,
            working_directory,
        }
    }

    fn collect_git_info() -> Option<GitInfo> {
        use std::process::Command;

        // Check if git is available and we're in a git repo
        let branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                } else {
                    None
                }
            })?;

        let status = Command::new("git")
            .args(["status", "--short"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|| "unknown".to_string());

        let recent_commits = Command::new("git")
            .args(["log", "--oneline", "-5"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().map(String::from).collect())
            .unwrap_or_else(Vec::new);

        Some(GitInfo {
            branch,
            status,
            recent_commits,
        })
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Schedule Context\n\n");
        md.push_str(&format!("**Time**: {}\n", self.current_time));
        md.push_str(&format!("**Directory**: {}\n\n", self.working_directory));

        if let Some(ref task) = self.current_task {
            md.push_str("## Current Task\n\n");
            md.push_str(&format!("- **Title**: {}\n", task.title));
            md.push_str(&format!("- **Time**: {} - {}\n", task.start_time, task.end_time));
            md.push_str(&format!("- **Status**: {}\n", task.status));
            if let Some(elapsed) = task.elapsed_minutes {
                md.push_str(&format!(
                    "- **Progress**: {}m / {}m\n",
                    elapsed, task.estimated_duration
                ));
            }
            md.push_str("\n");
        }

        if let Some(ref task) = self.next_task {
            md.push_str("## Next Task\n\n");
            md.push_str(&format!("- **Title**: {}\n", task.title));
            md.push_str(&format!("- **Time**: {} - {}\n\n", task.start_time, task.end_time));
        }

        md.push_str("## Today's Schedule\n\n");
        let summary = &self.today_schedule;
        md.push_str(&format!("- **Date**: {}\n", summary.date));
        md.push_str(&format!("- **Total Tasks**: {}\n", summary.total_tasks));
        md.push_str(&format!("- **Completed**: {}\n", summary.completed_tasks));
        md.push_str(&format!("- **In Progress**: {}\n", summary.in_progress_tasks));
        md.push_str(&format!("- **Pending**: {}\n", summary.pending_tasks));
        md.push_str(&format!(
            "- **Completion Rate**: {:.1}%\n",
            summary.completion_rate * 100.0
        ));
        md.push_str(&format!(
            "- **Time**: {}m estimated, {}m actual\n\n",
            summary.total_estimated_minutes, summary.total_actual_minutes
        ));

        if let Some(ref git) = self.git_info {
            md.push_str("## Git Info\n\n");
            md.push_str(&format!("- **Branch**: {}\n", git.branch));
            if !git.status.trim().is_empty() {
                md.push_str(&format!("- **Status**:\n```\n{}\n```\n", git.status));
            }
            if !git.recent_commits.is_empty() {
                md.push_str("- **Recent Commits**:\n");
                for commit in &git.recent_commits {
                    md.push_str(&format!("  - {}\n", commit));
                }
            }
        }

        md
    }
}

impl TaskInfo {
    fn from_task(task: &Task) -> Self {
        Self {
            title: task.title.clone(),
            start_time: task.start_time.format("%H:%M").to_string(),
            end_time: task.end_time.format("%H:%M").to_string(),
            status: format!("{:?}", task.status),
            elapsed_minutes: task.elapsed_minutes(),
            estimated_duration: task.estimated_duration_minutes,
        }
    }
}

impl ScheduleSummary {
    fn from_schedule(schedule: &Schedule) -> Self {
        let total_tasks = schedule.tasks.len();
        let completed_tasks = schedule
            .tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Completed))
            .count();
        let in_progress_tasks = schedule
            .tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::InProgress))
            .count();
        let pending_tasks = schedule
            .tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Pending))
            .count();

        let completion_rate = schedule.completion_rate();

        let total_estimated_minutes = schedule
            .tasks
            .iter()
            .map(|t| t.estimated_duration_minutes)
            .sum();

        let total_actual_minutes = schedule
            .tasks
            .iter()
            .filter_map(|t| t.actual_duration_minutes)
            .sum();

        Self {
            date: schedule.date.format("%Y-%m-%d").to_string(),
            total_tasks,
            completed_tasks,
            in_progress_tasks,
            pending_tasks,
            completion_rate,
            total_estimated_minutes,
            total_actual_minutes,
        }
    }
}
