use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::models::Schedule;
use crate::storage::{JsonStorage, Storage};

pub struct App {
    storage: JsonStorage,
    schedule: Option<Schedule>,
    selected_index: usize,
    should_quit: bool,
}

impl App {
    pub fn new(storage: JsonStorage) -> anyhow::Result<Self> {
        let schedule = storage.load_today()?;
        Ok(Self {
            storage,
            schedule,
            selected_index: 0,
            should_quit: false,
        })
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if self.should_quit {
                break;
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('r') => {
                // Reload schedule
                if let Ok(schedule) = self.storage.load_today() {
                    self.schedule = schedule;
                }
            }
            _ => {}
        }
    }

    fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn select_next(&mut self) {
        if let Some(ref schedule) = self.schedule {
            if self.selected_index < schedule.tasks.len().saturating_sub(1) {
                self.selected_index += 1;
            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Timeline
                Constraint::Percentage(35), // Details
                Constraint::Percentage(25), // Stats
            ])
            .split(f.size());

        self.render_timeline(f, chunks[0]);
        self.render_details(f, chunks[1]);
        self.render_stats(f, chunks[2]);
    }

    fn render_timeline(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Timeline ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        if let Some(ref schedule) = self.schedule {
            let items: Vec<ListItem> = schedule
                .tasks
                .iter()
                .map(|task| {
                    let time_str = format!(
                        "{} - {}",
                        task.start_time.format("%H:%M"),
                        task.end_time.format("%H:%M")
                    );

                    let status_icon = match task.status {
                        crate::models::TaskStatus::Completed => "✓",
                        crate::models::TaskStatus::InProgress => "▶",
                        crate::models::TaskStatus::Paused => "⏸",
                        crate::models::TaskStatus::Pending => "○",
                        crate::models::TaskStatus::Skipped => "⊘",
                    };

                    let status_color = match task.status {
                        crate::models::TaskStatus::Completed => Color::Green,
                        crate::models::TaskStatus::InProgress => Color::Yellow,
                        crate::models::TaskStatus::Paused => Color::Cyan,
                        crate::models::TaskStatus::Pending => Color::Gray,
                        crate::models::TaskStatus::Skipped => Color::Red,
                    };

                    let line = Line::from(vec![
                        Span::styled(status_icon, Style::default().fg(status_color)),
                        Span::raw(" "),
                        Span::styled(time_str, Style::default().fg(Color::Cyan)),
                        Span::raw(" "),
                        Span::raw(&task.title),
                    ]);

                    ListItem::new(line)
                })
                .collect();

            let mut state = ListState::default();
            state.select(Some(self.selected_index));

            let list = List::new(items)
                .block(block)
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("► ");

            f.render_stateful_widget(list, area, &mut state);
        } else {
            let paragraph = Paragraph::new("No schedule for today")
                .block(block)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(paragraph, area);
        }
    }

    fn render_details(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        if let Some(ref schedule) = self.schedule {
            if let Some(task) = schedule.tasks.get(self.selected_index) {
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled("Title: ", Style::default().fg(Color::Cyan)),
                        Span::raw(&task.title),
                    ]),
                    Line::from(vec![
                        Span::styled("Time: ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!(
                            "{} - {}",
                            task.start_time.format("%H:%M"),
                            task.end_time.format("%H:%M")
                        )),
                    ]),
                    Line::from(vec![
                        Span::styled("Duration: ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{}m", task.estimated_duration_minutes)),
                    ]),
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{:?}", task.status)),
                    ]),
                    Line::from(""),
                ];

                if let Some(elapsed) = task.elapsed_minutes() {
                    let progress =
                        (elapsed as f64 / task.estimated_duration_minutes as f64 * 100.0) as u32;
                    lines.push(Line::from(vec![
                        Span::styled("Progress: ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{}m / {}m ({}%)", elapsed, task.estimated_duration_minutes, progress)),
                    ]));
                }

                if let Some(ref pomodoro) = task.pomodoro {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("Pomodoro: ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!(
                            "{}/{}",
                            pomodoro.completed_pomodoros, pomodoro.total_pomodoros
                        )),
                    ]));
                }

                if !task.tags.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("Tags: ", Style::default().fg(Color::Cyan)),
                        Span::raw(task.tags.join(", ")),
                    ]));
                }

                if let Some(ref notes) = task.notes {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("Notes: ", Style::default().fg(Color::Cyan)),
                    ]));
                    lines.push(Line::from(notes.as_str()));
                }

                let paragraph = Paragraph::new(lines).block(block);
                f.render_widget(paragraph, area);
            } else {
                let paragraph = Paragraph::new("No task selected")
                    .block(block)
                    .style(Style::default().fg(Color::Gray));
                f.render_widget(paragraph, area);
            }
        } else {
            let paragraph = Paragraph::new("No schedule for today")
                .block(block)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(paragraph, area);
        }
    }

    fn render_stats(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Stats ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        if let Some(ref schedule) = self.schedule {
            let total = schedule.tasks.len();
            let completed = schedule
                .tasks
                .iter()
                .filter(|t| matches!(t.status, crate::models::TaskStatus::Completed))
                .count();
            let in_progress = schedule
                .tasks
                .iter()
                .filter(|t| matches!(t.status, crate::models::TaskStatus::InProgress))
                .count();
            let pending = schedule
                .tasks
                .iter()
                .filter(|t| matches!(t.status, crate::models::TaskStatus::Pending))
                .count();

            let completion_rate = schedule.completion_rate();

            let current_time = Local::now().format("%H:%M:%S").to_string();

            let lines = vec![
                Line::from(vec![
                    Span::styled("Time: ", Style::default().fg(Color::Cyan)),
                    Span::raw(current_time),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Total: ", Style::default().fg(Color::Cyan)),
                    Span::raw(total.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Completed: ", Style::default().fg(Color::Green)),
                    Span::raw(completed.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("In Progress: ", Style::default().fg(Color::Yellow)),
                    Span::raw(in_progress.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Pending: ", Style::default().fg(Color::Gray)),
                    Span::raw(pending.to_string()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Completion: ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{:.1}%", completion_rate * 100.0)),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Controls:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from("↑/k - Up"),
                Line::from("↓/j - Down"),
                Line::from("r - Reload"),
                Line::from("q/Esc - Quit"),
            ];

            let paragraph = Paragraph::new(lines).block(block);
            f.render_widget(paragraph, area);
        } else {
            let paragraph = Paragraph::new("No schedule for today")
                .block(block)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(paragraph, area);
        }
    }
}
