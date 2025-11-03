# Scheduler - Psychology-Based Daily Task Manager

A Rust-based daily scheduler built on 12 psychological principles to help you actually stick to your schedule.

## Features

### Core Scheduling
- **Time Block Management**: Create, edit, and track daily time blocks
- **Real-time Progress**: Live tracking of current and next tasks
- **Multiple Interfaces**: CLI, Terminal Widget, and Full TUI
- **Background Daemon**: Automatic time tracking and updates

### Psychological Principles (12 Methodologies)
1. **Implementation Intentions**: If-Then planning
2. **Time Blocking**: Dedicated time slots for tasks
3. **Pomodoro Technique**: 25/5/15 minute cycles
4. **Progress Monitoring**: Real-time completion tracking
5. **Time Awareness**: Estimated vs. actual time comparison
6. **Pre-commitment**: Plan ahead to reduce decision fatigue
7. **Zeigarnik Effect**: Track incomplete tasks
8. **Streaks**: Daily completion tracking with fire emoji rewards
9. **Social Accountability**: Share progress with Claude Code
10. **Time Accountability**: Earned/wasted time tracking with clear consequences
11. **Friction Reduction**: One-command task management
12. **Fresh Start Effect**: Daily reset for new beginnings

### Time Accountability System
- **Earned Time**: Time you successfully followed your plan
  - On-time completion: Full credit
  - Early completion: Bonus time! 🎉
  - Late completion: Penalty applied ⚠
- **Wasted Time**: Time from skipped or incomplete tasks
- **Efficiency Score**: (Earned / Planned) × 100%
- **Grades**: A+ to F based on performance
- **Immediate Feedback**: Instant results after each task

### AI Integration
- **Claude Code Integration**: Get schedule validation and optimization suggestions
- **Context Export**: Share your schedule with AI for personalized advice
- **Smart Templates**: Pre-built prompts for common scenarios

## Installation

### Prerequisites
- Rust 1.75 or higher
- Git (for installation from source)

### From Source

```bash
git clone <repository-url>
cd scheduler
cargo build --release
```

The binary will be available at `target/release/scheduler` (or `scheduler.exe` on Windows).

### Add to PATH

#### Windows
```powershell
# Add to your PATH environment variable
$env:Path += ";C:\path\to\scheduler\target\release"
```

#### macOS/Linux
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:/path/to/scheduler/target/release"
```

## Quick Start

### 1. Add your first task
```bash
sched add "Deep Work Session" --start 09:00 --end 11:00
sched add "Lunch Break" --start 11:00 --end 12:00
sched add "Code Review" --start 12:00 --end 13:00
```

### 2. Start working
```bash
# Start the current task
sched start

# Check status
sched status

# Complete when done
sched complete
```

### 3. View your progress
```bash
# Daily stats
sched stats

# Time accountability report
sched report

# Efficiency trend
sched efficiency

# Streak tracking
sched streak
```

### 4. Launch the TUI
```bash
# Full-screen interactive interface
sched ui
```

## Command Reference

### Task Management
- `sched add <title> --start HH:MM --end HH:MM` - Add a new task
- `sched list` - Show today's schedule
- `sched start` - Start the current task
- `sched pause` - Pause the current task
- `sched complete` - Complete the current task
- `sched delete <id>` - Delete a task
- `sched status` - Show current status

### Pomodoro Timer
- `sched pomodoro start` - Start a Pomodoro session
- `sched pomodoro complete` - Complete current Pomodoro
- `sched pomodoro status` - Show Pomodoro progress

### Statistics & Accountability
- `sched stats` - Show daily statistics
- `sched stats --week` - Show weekly statistics
- `sched report` - Time accountability report (today)
- `sched report --week` - Weekly accountability report
- `sched efficiency` - 7-day efficiency trend
- `sched efficiency --days 14` - Custom day range
- `sched streak` - Show completion streak

### Claude Integration
- `sched claude ask <question>` - Ask Claude for advice
- `sched claude validate` - Get schedule validation
- `sched claude optimize` - Get optimization suggestions
- `sched claude context` - Export schedule context

### UI & Daemon
- `sched ui` - Launch full-screen TUI
- `sched widget` - Show terminal widget
- `sched daemon start` - Start background daemon
- `sched daemon stop` - Stop background daemon
- `sched daemon status` - Check daemon status

## Configuration

Configuration file location:
- **Windows**: `%APPDATA%\scheduler\config.toml`
- **macOS/Linux**: `~/.config/scheduler/config.toml`

Default configuration:
```toml
default_time_block = 30
theme = "green"

[notifications]
task_start_reminder = true
task_end_reminder = true
reminder_minutes = 5

[daemon]
update_interval_seconds = 60
auto_start = true
```

### Theme Options
- `green` (default)
- `blue`
- `purple`
- `cyan`

## Data Storage

All data is stored locally in JSON format:
- **Windows**: `%APPDATA%\scheduler\scheduler\data\`
- **macOS/Linux**: `~/.local/share/scheduler/data/`

Files:
- `current.json` - Today's schedule
- `history/YYYY-MM-DD.json` - Historical schedules
- `streak.json` - Streak tracking data

## Examples

### Morning Routine
```bash
sched add "Morning Review" --start 08:00 --end 08:30
sched add "Priority Task" --start 08:30 --end 10:00
sched add "Email Processing" --start 10:00 --end 10:30
```

### Deep Work with Pomodoro
```bash
sched add "Write Documentation" --start 14:00 --end 16:00
sched start
sched pomodoro start
# Work for 25 minutes...
sched pomodoro complete
# Take a 5-minute break...
sched pomodoro start
# Continue...
```

### End of Day Review
```bash
sched report
sched efficiency
sched streak
```

### Weekly Planning with Claude
```bash
sched claude validate
# Copy the prompt and paste to Claude Code
# Get feedback on your schedule
```

## TUI Controls

When running `sched ui`:
- `↑/k` - Move up
- `↓/j` - Move down
- `r` - Reload schedule
- `q/Esc` - Quit

## Time Accountability Scoring

### Efficiency Score Formula
```
Efficiency = (Earned Time + Bonus Time - Penalty Time) / Planned Time × 100%
```

### Grades
- **A+ (95-100%)**: Outstanding! Exceptional time management
- **A (90-94%)**: Excellent performance
- **B (80-89%)**: Good, room for improvement
- **C (70-79%)**: Fair, needs attention
- **D (60-69%)**: Poor, significant issues
- **F (<60%)**: Failing, major problems

### Feedback Examples
- `+45m earned ✓` - Completed on time
- `+30m earned, +15m bonus! 🎉` - Finished early
- `+50m earned, -10m penalty ⚠` - Finished late
- `-60m wasted ✗` - Task skipped or incomplete

## Troubleshooting

### Daemon won't start
```bash
# Check if already running
sched daemon status

# Stop and restart
sched daemon stop
sched daemon start
```

### Data corruption
```bash
# Backup your data first
cp -r $APPDATA/scheduler/scheduler/data ~/backup

# Remove corrupted files
rm $APPDATA/scheduler/scheduler/data/current.json

# Restart the application
sched add "Recovery Task" --start 09:00 --end 10:00
```

### Widget not showing
Make sure your terminal supports ANSI colors and Unicode characters.

## Development

Built with:
- **Rust 1.86.0** (Edition 2021)
- **Ratatui 0.26.1** - Terminal UI
- **Crossterm 0.27.0** - Cross-platform terminal
- **Clap 4.5.1** - CLI parsing
- **Serde 1.0.197** - Serialization
- **Chrono 0.4.35** - Date/time handling

## Roadmap

### Completed (v1.0)
- ✅ Core scheduling and time blocking
- ✅ CLI interface
- ✅ Background daemon
- ✅ Terminal widget
- ✅ Full TUI
- ✅ Pomodoro timer
- ✅ Statistics and streaks
- ✅ Time accountability system
- ✅ Claude Code integration

### In Progress (v2.0) - GUI Desktop Application
- 🚧 **Tauri GUI Client (Windows)**
  - [ ] Visual timeline with drag & drop
  - [ ] System tray integration
  - [ ] Windows native notifications
  - [ ] Interactive dashboard
  - [ ] MSI installer
  - [ ] Auto-updater

### Future (v3.0)
- [ ] Schedule templates
- [ ] Recurring tasks
- [ ] Calendar integrations (Google Calendar, etc.)
- [ ] Team collaboration features
- [ ] Mobile notifications
- [ ] Habit tracking integration

## License

[To be determined]

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Acknowledgments

Built on psychological research from:
- B.F. Skinner (Reinforcement Theory)
- Peter Gollwitzer (Implementation Intentions)
- Daniel Kahneman & Amos Tversky (Loss Aversion)
- Bluma Zeigarnik (Zeigarnik Effect)
- Hengchen Dai & Katherine Milkman (Fresh Start Effect)
- BJ Fogg (Behavior Model)

## Support

For issues, questions, or suggestions:
- Open an issue on GitHub
- Check existing documentation
- Review the PRD and development roadmap

---

**Built with psychology in mind. Designed to help you actually stick to your schedule.**
