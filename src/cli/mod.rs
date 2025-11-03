pub mod commands;
pub mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sched")]
#[command(about = "Daily task scheduler with AI integration", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        title: String,
        #[arg(short, long)]
        start: String,
        #[arg(short, long)]
        end: String,
        #[arg(short, long)]
        tags: Option<String>,
        #[arg(short, long)]
        notes: Option<String>,
    },
    List,
    Start {
        id: Option<String>,
    },
    Pause,
    Complete,
    Status,
    Delete {
        id: String,
    },
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    Widget,
    /// Launch full-screen TUI interface
    Ui,
    Stats {
        #[arg(short, long)]
        week: bool,
    },
    Streak,
    Pomodoro {
        #[command(subcommand)]
        action: PomodoroAction,
    },
    Claude {
        #[command(subcommand)]
        action: ClaudeAction,
    },
    /// Show time accountability report
    Report {
        #[arg(short, long)]
        week: bool,
        #[arg(short, long)]
        month: bool,
    },
    /// Show time efficiency score trend
    Efficiency {
        #[arg(short, long)]
        days: Option<usize>,
    },
}

#[derive(Subcommand)]
pub enum DaemonAction {
    Start,
    Stop,
    Status,
}

pub use commands::execute_command;

#[derive(Subcommand)]
pub enum PomodoroAction {
    Start,
    Complete,
    Status,
}

#[derive(Subcommand)]
pub enum ClaudeAction {
    /// Ask Claude a question with current schedule context
    Ask {
        question: String,
    },
    /// Validate today's schedule
    Validate,
    /// Get optimization suggestions for remaining tasks
    Optimize {
        #[arg(short, long)]
        situation: Option<String>,
    },
    /// Export context as JSON or Markdown
    Context {
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
}
