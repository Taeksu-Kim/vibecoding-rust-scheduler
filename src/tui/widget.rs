use std::io;
use std::time::Duration;

use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};

use crate::models::TaskStatus;
use crate::storage::{JsonStorage, Storage};

pub fn run_widget() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let storage = JsonStorage::new()?;
    let mut should_quit = false;

    while !should_quit {
        terminal.draw(|f| {
            if let Err(e) = ui(f, &storage) {
                log::error!("UI draw error: {}", e);
            }
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    should_quit = true;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, storage: &JsonStorage) -> anyhow::Result<()> {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(size);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(chunks[1]);

    render_widget(f, storage, right_chunks[0])?;

    let info = Paragraph::new("Press 'q' to quit widget")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(info, chunks[0]);

    Ok(())
}

fn render_widget(f: &mut Frame, storage: &JsonStorage, area: Rect) -> anyhow::Result<()> {
    let schedule = storage.load_today()?;

    if let Some(schedule) = schedule {
        let block = Block::default()
            .title("🌱 Scheduler")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let completion = schedule.completion_rate();
        let current = schedule.get_current_task();
        let next = schedule.get_next_task();

        let content_area = block.inner(area);
        f.render_widget(block, area);

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .split(content_area);

        let now = Local::now().format("%H:%M").to_string();
        let header = Line::from(vec![
            Span::raw(now),
            Span::raw("  "),
            Span::styled(
                format!("{:.0}%", completion),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]);
        f.render_widget(Paragraph::new(header), inner_chunks[0]);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(completion / 100.0);
        f.render_widget(gauge, inner_chunks[1]);

        if let Some(task) = current {
            let elapsed = task.elapsed_minutes().unwrap_or(0);
            let status_icon = match task.status {
                TaskStatus::InProgress => "▶",
                _ => " ",
            };
            
            let current_text = vec![
                Line::from(Span::styled(
                    format!("{} {}", status_icon, task.title),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::raw(format!(
                    "  {}m / {}m",
                    elapsed, task.estimated_duration_minutes
                ))),
            ];
            let current_widget = Paragraph::new(current_text);
            f.render_widget(current_widget, inner_chunks[2]);
        } else {
            let idle = Paragraph::new("No active task")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(idle, inner_chunks[2]);
        }

        if let Some(task) = next {
            let next_text = vec![
                Line::from(Span::styled("⏱  Next:", Style::default().fg(Color::Yellow))),
                Line::from(Span::raw(format!("  {}", task.title))),
            ];
            let next_widget = Paragraph::new(next_text);
            f.render_widget(next_widget, inner_chunks[3]);
        }
    } else {
        let block = Block::default()
            .title("🌱 Scheduler")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let inner = block.inner(area);
        f.render_widget(block, area);
        
        let no_schedule = Paragraph::new("No schedule")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(no_schedule, inner);
    }

    Ok(())
}
