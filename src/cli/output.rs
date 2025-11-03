use colored::*;
use crate::models::{Task, TaskStatus, Schedule};

pub fn print_task(task: &Task) {
    let status_icon = match task.status {
        TaskStatus::Completed => "✓".green(),
        TaskStatus::InProgress => "▶".bright_green(),
        TaskStatus::Pending => " ".white(),
        TaskStatus::Paused => "⏸".yellow(),
        TaskStatus::Skipped => "⊘".red(),
    };

    let time_range = format!(
        "{} - {}",
        task.start_time.format("%H:%M"),
        task.end_time.format("%H:%M")
    );

    println!(
        "{} {} {} ({})",
        status_icon,
        time_range.cyan(),
        task.title.bold(),
        format!("{}min", task.estimated_duration_minutes).dimmed()
    );

    if let Some(notes) = &task.notes {
        println!("    {}", notes.dimmed());
    }

    if !task.tags.is_empty() {
        println!("    Tags: {}", task.tags.join(", ").blue());
    }

    if task.status == TaskStatus::InProgress {
        if let Some(elapsed) = task.elapsed_minutes() {
            let progress = if elapsed > task.estimated_duration_minutes {
                format!("{}min (OVERDUE)", elapsed).red()
            } else {
                format!("{}min / {}min", elapsed, task.estimated_duration_minutes).green()
            };
            println!("    Progress: {}", progress);
        }
    }
}

pub fn print_schedule(schedule: &Schedule) {
    println!("\n{}", "Today's Schedule".bold().underline());
    println!("{}\n", schedule.date.format("%Y-%m-%d (%A)").to_string().cyan());

    if schedule.tasks.is_empty() {
        println!("{}", "No tasks scheduled.".dimmed());
        return;
    }

    for task in &schedule.tasks {
        print_task(task);
        println!();
    }

    let completion = schedule.completion_rate();
    println!(
        "Completion: {}",
        format!("{:.1}%", completion).green().bold()
    );
}

pub fn success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}

pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}
