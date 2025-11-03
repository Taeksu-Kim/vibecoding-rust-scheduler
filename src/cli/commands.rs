use chrono::{Local, NaiveTime, TimeZone};
use colored::Colorize;
use std::collections::HashMap;

use crate::claude::{PromptTemplate, ScheduleContext};
use crate::daemon::{DaemonProcess, TimeTracker};
use crate::models::{Schedule, Task, TaskStatus};
use crate::storage::{JsonStorage, Storage};

use super::output;
use super::{ClaudeAction, Commands, DaemonAction};

pub fn execute_command(command: Commands) -> anyhow::Result<()> {
    let storage = JsonStorage::new()?;

    match command {
        Commands::Add {
            title,
            start,
            end,
            tags,
            notes,
        } => add_task(&storage, title, start, end, tags, notes),

        Commands::List => list_tasks(&storage),

        Commands::Start { id } => start_task(&storage, id),

        Commands::Pause => pause_task(&storage),

        Commands::Complete => complete_task(&storage),

        Commands::Status => show_status(&storage),

        Commands::Delete { id } => delete_task(&storage, id),

        Commands::Daemon { action } => daemon_command(action, storage),
        Commands::Widget => widget_command(),
        Commands::Ui => ui_command(storage),
        Commands::Stats { week } => stats_command(&storage, week),
        Commands::Streak => streak_command(&storage),
        Commands::Pomodoro { action } => pomodoro_command(&storage, action),
        Commands::Claude { action } => claude_command(&storage, action),
        Commands::Report { week, month } => report_command(&storage, week, month),
        Commands::Efficiency { days } => efficiency_command(&storage, days),
    }
}

fn daemon_command(action: DaemonAction, storage: JsonStorage) -> anyhow::Result<()> {
    let daemon = DaemonProcess::new()?;

    match action {
        DaemonAction::Start => {
            daemon.start()?;
            output::success("Daemon started");

            // Tracker 실행
            env_logger::init();
            let mut tracker = TimeTracker::new(storage);
            
            // CTRL+C 핸들러 (간단한 버전)
            ctrlc::set_handler(move || {
                log::info!("Received CTRL+C, shutting down...");
                std::process::exit(0);
            })?;

            tracker.start();
        }

        DaemonAction::Stop => {
            daemon.stop()?;
            output::success("Daemon stopped");
        }

        DaemonAction::Status => {
            if daemon.is_running() {
                output::info("Daemon is running");
            } else {
                output::info("Daemon is not running");
            }
        }
    }

    Ok(())
}

fn parse_time(time_str: &str) -> anyhow::Result<NaiveTime> {
    NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|_| anyhow::anyhow!("Invalid time format. Use HH:MM (e.g., 14:30)"))
}

fn add_task(
    storage: &JsonStorage,
    title: String,
    start_str: String,
    end_str: String,
    tags: Option<String>,
    notes: Option<String>,
) -> anyhow::Result<()> {
    let start_time = parse_time(&start_str)?;
    let end_time = parse_time(&end_str)?;

    let today = Local::now().date_naive();
    let start_datetime = Local.from_local_datetime(&today.and_time(start_time)).unwrap();
    let end_datetime = Local.from_local_datetime(&today.and_time(end_time)).unwrap();

    if end_datetime <= start_datetime {
        anyhow::bail!("End time must be after start time");
    }

    let mut task = Task::new(title.clone(), start_datetime, end_datetime);

    if let Some(tags_str) = tags {
        task.tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
    }

    task.notes = notes;

    let mut schedule = storage.load_today()?.unwrap_or_else(Schedule::today);
    
    schedule.add_task(task).map_err(|e| anyhow::anyhow!(e))?;
    schedule.sort_by_time();
    storage.save_schedule(&schedule)?;

    output::success(&format!("Task '{}' added successfully", title));
    Ok(())
}

fn list_tasks(storage: &JsonStorage) -> anyhow::Result<()> {
    let schedule = storage.load_today()?;

    match schedule {
        Some(mut s) => {
            s.sort_by_time();
            output::print_schedule(&s);
        }
        None => {
            output::info("No schedule for today. Use 'sched add' to create tasks.");
        }
    }

    Ok(())
}

fn start_task(storage: &JsonStorage, id: Option<String>) -> anyhow::Result<()> {
    let mut schedule = storage
        .load_today()?
        .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

    let task_id = if let Some(id) = id {
        id
    } else {
        schedule
            .get_next_task()
            .ok_or_else(|| anyhow::anyhow!("No pending tasks"))?
            .id
            .clone()
    };

    let task = schedule
        .find_task_mut(&task_id)
        .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
    
    let task_title = task.title.clone();
    task.start();
    
    storage.save_schedule(&schedule)?;

    output::success(&format!("Started task: {}", task_title));
    Ok(())
}

fn pause_task(storage: &JsonStorage) -> anyhow::Result<()> {
    let mut schedule = storage
        .load_today()?
        .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

    let current_id = schedule
        .get_current_task()
        .ok_or_else(|| anyhow::anyhow!("No task is currently in progress"))?
        .id
        .clone();

    let task = schedule.find_task_mut(&current_id).unwrap();
    let task_title = task.title.clone();
    task.pause();
    
    storage.save_schedule(&schedule)?;

    output::success(&format!("Paused task: {}", task_title));
    Ok(())
}

fn complete_task(storage: &JsonStorage) -> anyhow::Result<()> {
    use crate::models::TimeAccountability;

    let mut schedule = storage
        .load_today()?
        .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

    let current_id = schedule
        .get_current_task()
        .ok_or_else(|| anyhow::anyhow!("No task is currently in progress"))?
        .id
        .clone();

    let task = schedule.find_task_mut(&current_id).unwrap();
    let task_title = task.title.clone();
    task.complete();

    // Calculate time accountability
    let accountability = TimeAccountability::from_task(task);

    storage.save_schedule(&schedule)?;

    output::success(&format!("Completed task: {}", task_title));

    // Show immediate feedback
    if let Some(feedback) = accountability.feedback_message() {
        if accountability.bonus_time > 0 {
            println!("{}", feedback.green());
        } else if accountability.penalty_time > 0 {
            println!("{}", feedback.yellow());
        } else {
            println!("{}", feedback.green());
        }
    }

    let completion_rate = schedule.completion_rate();
    output::info(&format!("Today's completion: {:.1}%", completion_rate));

    Ok(())
}

fn show_status(storage: &JsonStorage) -> anyhow::Result<()> {
    let schedule = storage
        .load_today()?
        .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

    if let Some(current) = schedule.get_current_task() {
        println!("\n{}", "Current Task:".bold());
        output::print_task(current);
    } else {
        output::info("No task currently in progress");
    }

    if let Some(next) = schedule.get_next_task() {
        println!("\n{}", "Next Task:".bold());
        output::print_task(next);
    }

    let completion_rate = schedule.completion_rate();
    println!(
        "\n{}: {:.1}%",
        "Today's Progress".bold(),
        completion_rate
    );

    Ok(())
}

fn delete_task(storage: &JsonStorage, id: String) -> anyhow::Result<()> {
    let mut schedule = storage
        .load_today()?
        .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

    let task = schedule
        .remove_task(&id)
        .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

    storage.save_schedule(&schedule)?;

    output::success(&format!("Deleted task: {}", task.title));
    Ok(())
}

fn widget_command() -> anyhow::Result<()> {
    crate::tui::run_widget()
}

fn ui_command(storage: JsonStorage) -> anyhow::Result<()> {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = crate::tui::App::new(storage)?;
    let res = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn stats_command(storage: &JsonStorage, week: bool) -> anyhow::Result<()> {
    if week {
        show_weekly_stats(storage)
    } else {
        show_daily_stats(storage)
    }
}

fn show_daily_stats(storage: &JsonStorage) -> anyhow::Result<()> {
    let schedule = storage
        .load_today()?
        .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

    println!("\n{}", "📊 Daily Statistics".bold());
    println!("{}\n", Local::now().format("%Y-%m-%d (%A)").to_string().cyan());

    let completion = schedule.completion_rate();
    let completed = schedule.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
    let total = schedule.tasks.len();

    println!("{}: {}/{} ({:.1}%)", 
        "Tasks Completed".bold(), 
        completed, 
        total, 
        completion
    );

    let progress_bar = create_progress_bar(completion);
    println!("{}\n", progress_bar.green());

    if let Some(accuracy) = schedule.time_accuracy() {
        println!("{}: {:.1}%", "Time Accuracy".bold(), accuracy);
        let accuracy_bar = create_progress_bar(accuracy);
        println!("{}\n", accuracy_bar.blue());
    }

    let focus_time: i64 = schedule
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .filter_map(|t| t.actual_duration_minutes)
        .sum();

    println!("{}: {}h {}m", 
        "Focus Time".bold(), 
        focus_time / 60, 
        focus_time % 60
    );

    let pending = schedule.tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
    let paused = schedule.tasks.iter().filter(|t| t.status == TaskStatus::Paused).count();

    println!("{}: {}", "Pending".yellow(), pending);
    println!("{}: {}", "Paused".yellow(), paused);

    Ok(())
}

fn show_weekly_stats(storage: &JsonStorage) -> anyhow::Result<()> {
    use chrono::Duration;

    println!("\n{}", "📊 Weekly Statistics".bold());
    println!("{}\n", "Last 7 days".cyan());

    let today = Local::now();
    
    for i in (0..7).rev() {
        let date = today - Duration::days(i);
        let schedule = storage.load_schedule(date)?;

        let day_str = date.format("%a").to_string();
        
        if let Some(schedule) = schedule {
            let completion = schedule.completion_rate();
            let bar = create_progress_bar(completion);
            
            let indicator = if date.date_naive() == today.date_naive() {
                "◄── Today"
            } else {
                ""
            };

            println!("  {} {} {:.0}% {}", 
                day_str.cyan(), 
                bar.green(), 
                completion,
                indicator.yellow()
            );
        } else {
            println!("  {} {}", day_str.dimmed(), "No data".dimmed());
        }
    }

    Ok(())
}

fn streak_command(storage: &JsonStorage) -> anyhow::Result<()> {
    let streak = storage.load_streak()?;

    println!("\n{}", "🔥 Streak Information".bold());
    println!();

    println!("{}: {} days", 
        "Current Streak".bold().green(), 
        streak.current_streak
    );

    println!("{}: {} days", 
        "Best Streak".bold().yellow(), 
        streak.best_streak
    );

    println!("{}: {}", 
        "Last Updated".dimmed(), 
        streak.last_update.format("%Y-%m-%d %H:%M")
    );

    let fire_count = (streak.current_streak / 7).min(5) as usize;
    if fire_count > 0 {
        println!("\n{}", "🔥".repeat(fire_count));
    }

    Ok(())
}

fn create_progress_bar(percentage: f64) -> String {
    let filled = (percentage / 100.0 * 20.0) as usize;
    let empty = 20 - filled;
    
    format!("{}{}  {:.0}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage
    )
}

fn pomodoro_command(storage: &JsonStorage, action: super::PomodoroAction) -> anyhow::Result<()> {
    use super::PomodoroAction;
    use crate::models::PomodoroSession;

    match action {
        PomodoroAction::Start => {
            let mut schedule = storage
                .load_today()?
                .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

            let current = schedule
                .get_current_task()
                .ok_or_else(|| anyhow::anyhow!("No task is currently in progress"))?;

            let current_id = current.id.clone();
            let task = schedule.find_task_mut(&current_id).unwrap();

            if task.pomodoro.is_none() {
                task.pomodoro = Some(PomodoroSession::new(task.estimated_duration_minutes));
            }

            let pomodoro = task.pomodoro.as_mut().unwrap();
            pomodoro.start_pomodoro();

            // Extract info before save
            let task_title = task.title.clone();
            let current_pomodoro = pomodoro.completed_pomodoros + 1;
            let total_pomodoros = pomodoro.total_pomodoros;
            let duration = pomodoro.pomodoro_duration;

            storage.save_schedule(&schedule)?;

            output::success(&format!(
                "Started Pomodoro {}/{} for '{}'",
                current_pomodoro,
                total_pomodoros,
                task_title
            ));

            output::info(&format!("Focus for {} minutes", duration));
        }

        PomodoroAction::Complete => {
            let mut schedule = storage
                .load_today()?
                .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

            let current = schedule
                .get_current_task()
                .ok_or_else(|| anyhow::anyhow!("No task is currently in progress"))?;

            let current_id = current.id.clone();
            let task = schedule.find_task_mut(&current_id).unwrap();

            let pomodoro = task
                .pomodoro
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("No Pomodoro session active"))?;

            pomodoro.complete_pomodoro();

            // Extract info before save
            let completed = pomodoro.completed_pomodoros;
            let total = pomodoro.total_pomodoros;
            let is_complete = pomodoro.is_complete();
            let break_duration = pomodoro.next_break_duration();

            storage.save_schedule(&schedule)?;

            output::success(&format!(
                "Completed Pomodoro {}/{}",
                completed,
                total
            ));

            if is_complete {
                output::info("All Pomodoros completed! 🎉");
            } else {
                output::info(&format!("Take a {} minute break", break_duration));
            }
        }

        PomodoroAction::Status => {
            let schedule = storage
                .load_today()?
                .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

            let current = schedule
                .get_current_task()
                .ok_or_else(|| anyhow::anyhow!("No task is currently in progress"))?;

            if let Some(pomodoro) = &current.pomodoro {
                println!("\n{}", "🍅 Pomodoro Status".bold());
                println!("Task: {}", current.title.cyan());
                println!(
                    "Progress: {}/{}",
                    pomodoro.completed_pomodoros, pomodoro.total_pomodoros
                );

                if let Some(elapsed) = pomodoro.elapsed_minutes() {
                    let remaining = pomodoro.remaining_minutes().unwrap();
                    println!(
                        "Current: {}m elapsed, {}m remaining",
                        elapsed, remaining
                    );
                }

                let bar = create_progress_bar(
                    (pomodoro.completed_pomodoros as f64 / pomodoro.total_pomodoros as f64) * 100.0,
                );
                println!("{}", bar.green());
            } else {
                output::info("No Pomodoro session. Use 'sched pomodoro start'");
            }
        }
    }

    Ok(())
}

fn claude_command(storage: &JsonStorage, action: ClaudeAction) -> anyhow::Result<()> {
    use super::ClaudeAction;

    let schedule = storage
        .load_today()?
        .ok_or_else(|| anyhow::anyhow!("No schedule found for today"))?;

    let context = ScheduleContext::collect(&schedule);

    match action {
        ClaudeAction::Ask { question } => {
            let template = PromptTemplate::task_assistant();
            let mut vars = HashMap::new();
            vars.insert("context".to_string(), context.to_markdown());
            vars.insert("question".to_string(), question);

            let prompt = template.render(&vars);

            println!("\n{}\n", "=".repeat(80).bright_blue());
            println!("{}", "Claude Prompt".bright_cyan().bold());
            println!("{}\n", "=".repeat(80).bright_blue());
            println!("{}", prompt);
            println!("\n{}\n", "=".repeat(80).bright_blue());

            output::info("Copy the prompt above and paste it to Claude Code");
        }

        ClaudeAction::Validate => {
            let template = PromptTemplate::schedule_validation();
            let mut vars = HashMap::new();
            vars.insert("context".to_string(), context.to_markdown());

            let prompt = template.render(&vars);

            println!("\n{}\n", "=".repeat(80).bright_blue());
            println!("{}", "Schedule Validation Prompt".bright_cyan().bold());
            println!("{}\n", "=".repeat(80).bright_blue());
            println!("{}", prompt);
            println!("\n{}\n", "=".repeat(80).bright_blue());

            output::info("Copy the prompt above and paste it to Claude Code");
        }

        ClaudeAction::Optimize { situation } => {
            let template = PromptTemplate::optimization();
            let mut vars = HashMap::new();
            vars.insert("context".to_string(), context.to_markdown());
            vars.insert(
                "situation".to_string(),
                situation.unwrap_or_else(|| "Running behind schedule".to_string()),
            );

            let prompt = template.render(&vars);

            println!("\n{}\n", "=".repeat(80).bright_blue());
            println!("{}", "Optimization Prompt".bright_cyan().bold());
            println!("{}\n", "=".repeat(80).bright_blue());
            println!("{}", prompt);
            println!("\n{}\n", "=".repeat(80).bright_blue());

            output::info("Copy the prompt above and paste it to Claude Code");
        }

        ClaudeAction::Context { format } => {
            match format.to_lowercase().as_str() {
                "json" => {
                    println!("{}", context.to_json()?);
                }
                "markdown" | "md" => {
                    println!("{}", context.to_markdown());
                }
                _ => {
                    output::error(&format!("Unknown format: {}", format));
                    output::info("Supported formats: json, markdown");
                }
            }
        }
    }

    Ok(())
}

fn report_command(storage: &JsonStorage, week: bool, month: bool) -> anyhow::Result<()> {
    use crate::models::DailyAccountability;
    use chrono::Datelike;

    if month {
        output::info("Monthly report not yet implemented");
        return Ok(());
    }

    if week {
        // Weekly report
        let today = Local::now().date_naive();
        let mut total_planned = 0i64;
        let mut total_earned = 0i64;
        let mut total_wasted = 0i64;
        let mut total_bonus = 0i64;
        let mut total_penalty = 0i64;

        println!("\n{}", "Weekly Time Accountability Report".bold().green());
        println!("{}", "=".repeat(50));

        for days_ago in (0..7).rev() {
            let date = today - chrono::Duration::days(days_ago);
            let date_time = Local
                .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
                .unwrap();

            if let Ok(Some(schedule)) = storage.load_schedule(date_time) {
                let daily = DailyAccountability::from_tasks(date_time, &schedule.tasks);

                total_planned += daily.total_planned;
                total_earned += daily.total_earned;
                total_wasted += daily.total_wasted;
                total_bonus += daily.total_bonus;
                total_penalty += daily.total_penalty;

                let score = daily.efficiency_score();
                let grade = daily.grade();
                let _score_color = if score >= 90.0 {
                    "green"
                } else if score >= 70.0 {
                    "yellow"
                } else {
                    "red"
                };

                println!(
                    "{}: Efficiency {:.1}% ({}) | Earned: {}m | Wasted: {}m",
                    date.format("%Y-%m-%d"),
                    score,
                    grade,
                    daily.net_earned(),
                    daily.total_wasted
                );
            }
        }

        println!("{}", "=".repeat(50));
        let week_score = if total_planned > 0 {
            ((total_earned + total_bonus - total_penalty) as f64 / total_planned as f64) * 100.0
        } else {
            0.0
        };

        println!("\n{}", "Week Summary:".bold());
        println!("  Total Planned: {}m", total_planned);
        println!("  Total Earned: {}m", total_earned);
        println!("  Total Bonus: {}m", total_bonus.to_string().green());
        println!("  Total Penalty: {}m", total_penalty.to_string().yellow());
        println!("  Total Wasted: {}m", total_wasted.to_string().red());
        println!("\n  Overall Efficiency: {:.1}%", week_score);

    } else {
        // Daily report
        let schedule = storage
            .load_today()?
            .ok_or_else(|| anyhow::anyhow!("No schedule found"))?;

        let daily = DailyAccountability::from_tasks(Local::now(), &schedule.tasks);

        println!("\n{}", "Daily Time Accountability Report".bold().green());
        println!("{}", "=".repeat(50));
        println!("Date: {}", daily.date.format("%Y-%m-%d"));
        println!();
        println!("  Total Planned: {}m ({:.1}h)", daily.total_planned, daily.total_planned as f64 / 60.0);
        println!("  {} {}m ({:.1}h)", "Earned:".green(), daily.total_earned, daily.total_earned as f64 / 60.0);

        if daily.total_bonus > 0 {
            println!("  {} +{}m 🎉", "Bonus:".bright_green(), daily.total_bonus);
        }
        if daily.total_penalty > 0 {
            println!("  {} -{}m ⚠", "Penalty:".yellow(), daily.total_penalty);
        }
        if daily.total_wasted > 0 {
            println!("  {} -{}m ✗", "Wasted:".red(), daily.total_wasted);
        }

        println!();
        println!("  {} {}m ({:.1}h)", "Net Earned:".bold(), daily.net_earned(), daily.net_earned() as f64 / 60.0);
        println!("  {} {:.1}% ({})", "Efficiency Score:".bold(), daily.efficiency_score(), daily.grade());

        // Task breakdown
        println!("\n{}", "Task Breakdown:".bold());
        println!("{}", "-".repeat(50));

        for task in &schedule.tasks {
            use crate::models::TimeAccountability;
            let perf = TimeAccountability::from_task(task);

            if let Some(msg) = perf.feedback_message() {
                let colored_msg = if perf.bonus_time > 0 {
                    msg.green()
                } else if perf.penalty_time > 0 {
                    msg.yellow()
                } else if perf.wasted_time > 0 {
                    msg.red()
                } else {
                    msg.green()
                };
                println!("  {} - {}", task.title, colored_msg);
            }
        }
    }

    Ok(())
}

fn efficiency_command(storage: &JsonStorage, days: Option<usize>) -> anyhow::Result<()> {
    use crate::models::DailyAccountability;
    use chrono::Datelike;

    let num_days = days.unwrap_or(7);
    let today = Local::now().date_naive();

    println!("\n{}", format!("{}-Day Efficiency Trend", num_days).bold().cyan());
    println!("{}", "=".repeat(60));

    let mut scores: Vec<(String, f64)> = Vec::new();

    for days_ago in (0..num_days).rev() {
        let date = today - chrono::Duration::days(days_ago as i64);
        let date_time = Local
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .unwrap();

        if let Ok(Some(schedule)) = storage.load_schedule(date_time) {
            let daily = DailyAccountability::from_tasks(date_time, &schedule.tasks);
            let score = daily.efficiency_score();
            scores.push((date.format("%m/%d").to_string(), score));

            // ASCII bar chart
            let bar_length = (score / 100.0 * 40.0) as usize;
            let bar = "█".repeat(bar_length);

            let colored_bar = if score >= 90.0 {
                bar.green()
            } else if score >= 70.0 {
                bar.yellow()
            } else {
                bar.red()
            };

            println!("{} | {:<40} {:.1}% ({})",
                date.format("%m/%d"),
                colored_bar,
                score,
                daily.grade()
            );
        } else {
            println!("{} | {} (no data)", date.format("%m/%d"), "░".repeat(40).bright_black());
        }
    }

    println!("{}", "=".repeat(60));

    // Calculate average
    if !scores.is_empty() {
        let avg = scores.iter().map(|(_, s)| s).sum::<f64>() / scores.len() as f64;
        println!("Average Efficiency: {:.1}%", avg);

        // Trend
        if scores.len() >= 2 {
            let recent_avg = scores[scores.len().saturating_sub(3)..].iter().map(|(_, s)| s).sum::<f64>()
                / scores.len().saturating_sub(3).max(1) as f64;
            let early_avg = scores[..scores.len().saturating_sub(3).max(1)].iter().map(|(_, s)| s).sum::<f64>()
                / scores.len().saturating_sub(3).max(1) as f64;

            if recent_avg > early_avg + 5.0 {
                output::success("Improving trend! 📈");
            } else if recent_avg < early_avg - 5.0 {
                output::error("Declining trend 📉");
            } else {
                output::info("Stable performance");
            }
        }
    }

    Ok(())
}
