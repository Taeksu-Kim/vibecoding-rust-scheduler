# Claude Instructions
# Daily Scheduler 프로젝트 지침서

이 문서는 Claude가 이 프로젝트를 개발할 때 참고해야 할 전역 지침입니다.

---

## 프로젝트 개요

**Daily Scheduler**: 하루 단위 시간 관리에 특화된 Rust 기반 스케줄러
- 심리학 이론과 시간 관리 방법론 기반
- Claude Code와 자연스럽게 통합
- Terminal-first, Developer-friendly

---

## 핵심 문서

### 1. PRD.md
- 제품 요구사항 정의
- 12가지 심리학 방법론
- UI/UX 디자인 (4계층 하이브리드 아키텍처)
- Claude Code 연동 방식
- **개발 시작 전 반드시 확인**

### 2. DEVELOPMENT_ROADMAP.md
- 11개 Phase로 구성된 개발 계획
- Phase 0-10: 구체적인 할 일 체크리스트
- v1.0 MVP 범위 정의
- **현재 어느 Phase인지 항상 확인**

### 3. 이 문서 (CLAUDE.md)
- 개발 시 지켜야 할 코딩 규칙
- Rust 베스트 프랙티스
- 프로젝트별 컨벤션

---

## Dependencies & Versions

이 프로젝트에서 사용할 라이브러리와 버전입니다. 각 Phase별로 필요한 의존성이 다릅니다.

### Phase별 의존성 맵

| Phase | 라이브러리 | 용도 |
|-------|-----------|------|
| 0-1   | serde, chrono, anyhow, thiserror | 기본 데이터 모델 |
| 2     | clap, colored | CLI 인터페이스 |
| 3     | tokio (optional), interprocess | Daemon & IPC |
| 4     | ratatui, crossterm | TUI 위젯 |
| 5-6   | 추가 없음 | 기존 라이브러리 활용 |
| 7     | reqwest, serde_json | Claude API 연동 |
| 8-9   | 추가 없음 | 기존 라이브러리 활용 |
| 10    | notify-rust (optional) | OS 알림 |

### 전체 Cargo.toml (v1.0 기준)

```toml
[package]
name = "scheduler"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"  # 최소 Rust 버전

[dependencies]
# === Core (Phase 0-1) ===

# Serialization
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.114"

# Time handling
chrono = { version = "0.4.35", features = ["serde"] }

# Error handling
anyhow = "1.0.80"
thiserror = "1.0.57"

# === CLI (Phase 2) ===

# Command-line parsing
clap = { version = "4.5.1", features = ["derive", "cargo"] }

# Terminal colors
colored = "2.1.0"

# === Daemon (Phase 3) ===

# Async runtime (optional - 필요시만)
# tokio = { version = "1.36.0", features = ["full"] }

# IPC (Inter-Process Communication)
interprocess = "2.1.0"

# Process management
daemonize = "0.5.0"  # Unix only
# Windows service 필요시: windows-service = "0.6.0"

# === TUI (Phase 4) ===

# Terminal UI
ratatui = "0.26.1"
crossterm = "0.27.0"

# === Claude Integration (Phase 7) ===

# HTTP client
reqwest = { version = "0.11.24", features = ["json", "rustls-tls"], default-features = false }

# URL encoding
urlencoding = "2.1.3"

# === Utilities (All phases) ===

# Logging
log = "0.4.20"
env_logger = "0.11.2"

# Configuration
toml = "0.8.10"

# Path handling
directories = "5.0.1"  # 크로스플랫폼 config 경로

# UUID generation
uuid = { version = "1.7.0", features = ["v4", "serde"] }

# === Optional (Phase 10) ===

# OS notifications (optional)
# notify-rust = "4.10.0"

# === Dev Dependencies ===

[dev-dependencies]
# Testing
tempfile = "3.10.0"  # 임시 파일/디렉토리
assert_cmd = "2.0.13"  # CLI 테스트
predicates = "3.1.0"  # 테스트 assertions
mockall = "0.12.1"  # Mocking (필요시)

# Benchmarking
criterion = "0.5.1"

[[bench]]
name = "schedule_benchmark"
harness = false

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true  # 바이너리 크기 줄이기
```

### 주요 라이브러리 설명

#### 1. serde (1.0.197)
- **용도**: 직렬화/역직렬화 (JSON ↔ Rust 구조체)
- **버전 이유**: 1.0은 안정적이고 후방 호환성 보장
- **features**: `derive` - 자동 Serialize/Deserialize 구현

```rust
#[derive(Serialize, Deserialize)]
struct Task { }
```

#### 2. chrono (0.4.35)
- **용도**: 날짜/시간 처리
- **버전 이유**: 0.4는 성숙한 버전, 0.5는 아직 breaking changes 가능성
- **주의**: `chrono::Local::now()` 사용 시 TZ 관련 이슈 있을 수 있음
- **features**: `serde` - DateTime 직렬화 지원

```rust
use chrono::{DateTime, Local, Duration};
let now = Local::now();
```

#### 3. clap (4.5.1)
- **용도**: CLI 파싱
- **버전 이유**: 4.x는 최신 안정화 버전, derive 매크로 강력
- **features**: `derive` - 구조체 기반 CLI 정의

```rust
#[derive(Parser)]
struct Cli { }
```

#### 4. ratatui (0.26.1)
- **용도**: Terminal UI
- **버전 이유**: tui-rs의 후속 프로젝트, 활발한 개발
- **주의**: 자주 업데이트되므로 0.26.x에서 고정 추천

#### 5. crossterm (0.27.0)
- **용도**: 크로스플랫폼 터미널 제어
- **버전 이유**: ratatui와 호환성 최고
- **장점**: Windows/Linux/macOS 모두 지원

#### 6. reqwest (0.11.24)
- **용도**: HTTP 클라이언트 (Claude API 호출)
- **features**:
  - `json` - JSON body 지원
  - `rustls-tls` - OpenSSL 대신 Rust native TLS (빌드 편함)
- **주의**: `default-features = false`로 불필요한 것 제거

```rust
let client = reqwest::Client::new();
let response = client.post(url).json(&body).send().await?;
```

#### 7. interprocess (2.1.0)
- **용도**: IPC (CLI ↔ Daemon 통신)
- **장점**: 크로스플랫폼 (Unix socket, Windows named pipe)
- **대안**:
  - Unix only: `tokio::net::UnixStream`
  - Windows: `named_pipe`

#### 8. anyhow (1.0.80)
- **용도**: 간단한 에러 전파 (main, prototypes)
- **장점**: `?` 연산자로 모든 에러 타입 자동 변환

```rust
fn main() -> anyhow::Result<()> {
    // 편리한 에러 처리
}
```

#### 9. thiserror (1.0.57)
- **용도**: 커스텀 에러 타입 정의
- **장점**: derive 매크로로 간단히 Error trait 구현

```rust
#[derive(Error, Debug)]
enum MyError {
    #[error("not found")]
    NotFound,
}
```

#### 10. directories (5.0.1)
- **용도**: OS별 config/data 경로
- **예**:
  - Linux: `~/.config/scheduler/`
  - macOS: `~/Library/Application Support/scheduler/`
  - Windows: `%APPDATA%\scheduler\`

```rust
let config_dir = directories::ProjectDirs::from("com", "myapp", "scheduler")
    .unwrap()
    .config_dir();
```

### 버전 선택 기준

1. **Major version >= 1.0**: API 안정성 보장
2. **활발한 유지보수**: 최근 6개월 내 업데이트
3. **크로스플랫폼 지원**: Windows, Linux, macOS
4. **문서화**: docs.rs에 풍부한 예제
5. **커뮤니티**: GitHub stars, crates.io 다운로드

### 버전 업데이트 정책

```bash
# 마이너 버전 업데이트 (안전)
cargo update -p serde

# 메이저 버전은 신중하게
# Breaking changes 확인 후 수동으로 Cargo.toml 수정
```

### Phase별 의존성 추가 가이드

#### Phase 0-1: 기본 setup
```toml
[dependencies]
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.114"
chrono = { version = "0.4.35", features = ["serde"] }
anyhow = "1.0.80"
thiserror = "1.0.57"
uuid = { version = "1.7.0", features = ["v4", "serde"] }
directories = "5.0.1"
```

#### Phase 2: CLI 추가
```toml
clap = { version = "4.5.1", features = ["derive", "cargo"] }
colored = "2.1.0"
```

#### Phase 3: Daemon 추가
```toml
interprocess = "2.1.0"
daemonize = "0.5.0"
log = "0.4.20"
env_logger = "0.11.2"
```

#### Phase 4: TUI 추가
```toml
ratatui = "0.26.1"
crossterm = "0.27.0"
```

#### Phase 7: Claude 연동 추가
```toml
reqwest = { version = "0.11.24", features = ["json", "rustls-tls"], default-features = false }
urlencoding = "2.1.3"
toml = "0.8.10"  # config 파일용
```

### 호환성 매트릭스

| Rust Version | 지원 여부 | 비고 |
|--------------|---------|------|
| 1.75+        | ✅ Full | 권장 (Edition 2021) |
| 1.70-1.74    | ⚠️ Partial | 일부 feature 불가 |
| < 1.70       | ❌ No | 지원 안 함 |

| OS | 지원 여부 | 비고 |
|----|---------|------|
| Linux | ✅ Full | Primary target |
| macOS | ✅ Full | 완전 지원 |
| Windows | ⚠️ Partial | Daemon 기능 제한적 (WSL 권장) |

### 대안 라이브러리 (필요시 교체 가능)

| 용도 | 현재 선택 | 대안 |
|------|----------|------|
| CLI | clap | structopt (deprecated), argh |
| TUI | ratatui | cursive, tui-rs (archived) |
| Time | chrono | time (더 빠름, 하지만 API 복잡) |
| HTTP | reqwest | ureq (동기), hyper (저수준) |
| JSON | serde_json | simd-json (빠름, unsafe) |

### Cargo.lock 관리

```bash
# Cargo.lock은 git에 포함 (바이너리 프로젝트)
git add Cargo.lock

# 의존성 정확한 버전 보장
```

---

## GUI: Tauri (Phase 11)

**결정**: Tauri 사용 (v1.5.4 - 안정 버전)

### 의존성

```toml
[dependencies]
tauri = { version = "1.5.4", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
tauri-build = { version = "1.5.1", features = [] }
```

### Tauri Commands 예시

```rust
#[tauri::command]
fn get_schedule() -> Result<Schedule, String> {
    // scheduler 로직
}

#[tauri::command]
fn add_task(title: String, start: String, end: String) -> Result<String, String> {
    // Task 추가
}

#[tauri::command]
async fn ask_claude(question: String) -> Result<String, String> {
    // Claude API
}
```

### 프로젝트 구조

```
scheduler/
├── src/                    # Rust 백엔드
├── src-tauri/              # Tauri 앱
│   ├── src/main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── frontend/               # 프론트엔드 (Svelte/Vanilla)
└── Cargo.toml             # 워크스페이스
```

---

## Rust 개발 지침

### 1. 코드 스타일

#### Naming Conventions
```rust
// 모듈: snake_case
mod time_management;

// 구조체/Enum: PascalCase
struct TimeBlock { }
enum TaskStatus { }

// 함수/변수: snake_case
fn calculate_completion_rate() { }
let task_count = 10;

// 상수: SCREAMING_SNAKE_CASE
const MAX_TASKS_PER_DAY: usize = 20;

// Lifetime: 짧고 의미있게
fn foo<'a, 'task>(task: &'task Task) { }
```

#### 파일 구조
```rust
// 파일 상단: 모듈 선언 순서
// 1. extern crate (Rust 2018+에서는 거의 불필요)
// 2. use 선언 (std → external → crate)
// 3. 모듈 선언
// 4. 구조체/타입 정의
// 5. impl 블록
// 6. 함수

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::models::Task;

pub struct Schedule { }

impl Schedule { }

pub fn helper_function() { }
```

### 2. Error Handling

#### 기본 원칙
```rust
// ✅ Result를 사용한 에러 전파
fn load_schedule() -> Result<Schedule, SchedulerError> {
    let data = std::fs::read_to_string(path)?;
    let schedule = serde_json::from_str(&data)?;
    Ok(schedule)
}

// ✅ thiserror로 커스텀 에러 정의
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Invalid time range")]
    InvalidTimeRange,

    #[error("Storage error: {0}")]
    StorageError(#[from] std::io::Error),
}

// ✅ anyhow는 main이나 프로토타입에만
fn main() -> anyhow::Result<()> {
    // ...
}

// ❌ unwrap/expect는 정말 확신할 때만
let config = load_config().unwrap(); // 피하기

// ✅ 대신 이렇게
let config = load_config()
    .expect("Config file must exist at startup");
```

### 3. Ownership & Borrowing

#### 기본 가이드라인
```rust
// ✅ 소유권이 필요하지 않으면 빌려쓰기
fn print_task(task: &Task) {  // Task를 소비하지 않음
    println!("{}", task.title);
}

// ✅ 수정이 필요하면 mutable reference
fn update_task(task: &mut Task) {
    task.status = TaskStatus::InProgress;
}

// ✅ 소유권이 필요하면 명시적으로
fn consume_task(task: Task) {
    // task는 여기서 소비됨
}

// ✅ Clone은 명확히 필요할 때만
let task_copy = task.clone(); // 비용이 있음을 인지

// ✅ Cow를 활용한 최적화 (필요시)
use std::borrow::Cow;

fn process_title(title: Cow<str>) -> String {
    if title.contains("urgent") {
        title.into_owned() // 소유권 가져옴
    } else {
        title.into_owned()
    }
}
```

### 4. Collections & Iterators

```rust
// ✅ Iterator를 적극 활용
let completed_tasks: Vec<_> = tasks
    .iter()
    .filter(|t| t.status == TaskStatus::Completed)
    .collect();

// ✅ 성능이 중요하면 capacity 미리 할당
let mut tasks = Vec::with_capacity(20);

// ✅ 함수형 스타일 선호
let total_time: Duration = tasks
    .iter()
    .filter_map(|t| t.actual_duration)
    .sum();

// ❌ 불필요한 clone 피하기
for task in tasks.iter() {  // ✅
    // ...
}

for task in tasks.clone().iter() {  // ❌
    // ...
}
```

### 5. Option & Result 처리

```rust
// ✅ ? 연산자 활용
fn get_current_task(&self) -> Option<&Task> {
    self.tasks.iter().find(|t| t.is_current())
}

// ✅ map/and_then 체이닝
let duration = task
    .actual_duration
    .map(|d| d.as_secs())
    .unwrap_or(0);

// ✅ if let / match 활용
if let Some(task) = schedule.get_current_task() {
    println!("Current: {}", task.title);
}

match task.status {
    TaskStatus::Completed => { /* ... */ }
    TaskStatus::InProgress => { /* ... */ }
    _ => { /* ... */ }
}

// ✅ ok_or/ok_or_else로 Option → Result 변환
let task = schedule
    .find_task(&id)
    .ok_or(SchedulerError::TaskNotFound(id))?;
```

### 6. Async (필요시)

```rust
// Daemon이나 네트워크 통신에서 사용 가능

// ✅ tokio 사용 시
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ...
}

// ✅ async 함수
async fn fetch_claude_response(prompt: &str) -> Result<String, Error> {
    // ...
}

// ⚠️ 하지만 이 프로젝트는 일단 동기 코드로 시작
// 필요시에만 추가
```

### 7. Testing

```rust
// ✅ 각 모듈에 tests 모듈
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test", start, end);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_time_conflict() {
        let mut schedule = Schedule::new();
        schedule.add_task(task1).unwrap();

        // 시간 충돌 검증
        assert!(schedule.add_task(task2).is_err());
    }
}

// ✅ Integration tests (tests/ 디렉토리)
// tests/cli_test.rs
#[test]
fn test_add_command() {
    // CLI 전체 플로우 테스트
}
```

### 8. Documentation

```rust
// ✅ Public API는 문서화
/// 스케줄에 새로운 작업을 추가합니다.
///
/// # Arguments
///
/// * `task` - 추가할 작업
///
/// # Returns
///
/// 성공 시 작업 ID, 시간 충돌 시 에러
///
/// # Examples
///
/// ```
/// let schedule = Schedule::new();
/// let task = Task::new("Meeting", start, end);
/// schedule.add_task(task)?;
/// ```
pub fn add_task(&mut self, task: Task) -> Result<String, SchedulerError> {
    // ...
}

// ✅ 복잡한 로직에는 인라인 주석
// Calculate completion rate using actual vs estimated time
let rate = if estimated > 0 {
    (actual as f64 / estimated as f64) * 100.0
} else {
    0.0
};
```

### 9. Serialization (Serde)

```rust
use serde::{Deserialize, Serialize};

// ✅ 기본 derive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,

    // ✅ Option 필드는 skip_serializing_if
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    // ✅ DateTime는 그대로 직렬화 (chrono가 지원)
    pub start_time: DateTime<Local>,

    // ✅ 기본값 제공
    #[serde(default)]
    pub tags: Vec<String>,
}

// ✅ 커스텀 직렬화가 필요하면
#[serde(serialize_with = "serialize_duration")]
pub duration: Duration,
```

### 10. Performance Tips

```rust
// ✅ 문자열 처리
// 소유권 불필요하면 &str
fn process(s: &str) { }

// 빌드가 필요하면 String
fn build_message(name: &str) -> String {
    format!("Hello, {}", name)
}

// ✅ Vec vs Slice
fn process_tasks(tasks: &[Task]) {  // slice가 더 유연
    // ...
}

// ✅ 불필요한 할당 피하기
// ❌
let msg = format!("Task: {}", task.title);
println!("{}", msg);

// ✅
println!("Task: {}", task.title);

// ✅ Lazy evaluation
use std::sync::LazyLock;

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    load_config().expect("Failed to load config")
});
```

---

## 프로젝트별 컨벤션

### 1. 모듈 구조

```
src/
├── main.rs           # CLI 엔트리포인트
├── lib.rs            # 라이브러리 루트
├── models/           # 데이터 모델
│   ├── mod.rs
│   ├── task.rs
│   ├── schedule.rs
│   └── stats.rs
├── storage/          # 영속성
│   ├── mod.rs
│   └── json_storage.rs
├── daemon/           # 백그라운드 데몬
│   ├── mod.rs
│   └── time_tracker.rs
├── cli/              # CLI 인터페이스
│   ├── mod.rs
│   ├── commands.rs
│   └── output.rs
├── tui/              # TUI 위젯
│   ├── mod.rs
│   ├── widget.rs
│   └── full_ui.rs
└── claude/           # Claude 연동
    ├── mod.rs
    ├── client.rs
    └── prompts.rs
```

### 2. 에러 타입

```rust
// 각 모듈별로 에러 정의
pub enum SchedulerError { }
pub enum StorageError { }
pub enum DaemonError { }
pub enum ClaudeError { }

// 필요시 From trait 구현
impl From<StorageError> for SchedulerError {
    fn from(err: StorageError) -> Self {
        SchedulerError::Storage(err)
    }
}
```

### 3. 설정 관리

```rust
// config.rs
#[derive(Debug, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub default_block_size: Duration,
    pub theme: ColorTheme,
    pub claude_api_key: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        // ~/.config/scheduler/config.toml
    }
}
```

### 4. Time Handling

```rust
use chrono::{DateTime, Local, Duration};

// ✅ 항상 Local timezone 사용
let now = Local::now();

// ✅ Duration은 chrono::Duration
let duration = Duration::hours(2);

// ✅ 시간 파싱
fn parse_time(s: &str) -> Result<DateTime<Local>, ParseError> {
    // "14:00" → DateTime
}
```

---

## CLI 개발 가이드

### Clap 사용

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sched")]
#[command(about = "Daily task scheduler with AI integration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        title: String,
        #[arg(short, long)]
        start: String,
        #[arg(short, long)]
        end: String,
    },

    /// List today's schedule
    List,

    /// Start a task
    Start {
        /// Task ID (optional, starts next task if not provided)
        id: Option<String>,
    },
}
```

### 출력 포맷

```rust
// colored crate 사용
use colored::*;

println!("{} Task completed!", "✓".green());
println!("{} Time exceeded", "⚠".yellow());
println!("{} Error: ...", "✗".red());
```

---

## TUI 개발 가이드

### Ratatui 기본

```rust
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
};

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    loop {
        terminal.draw(|f| {
            // UI 렌더링
        })?;

        // 이벤트 처리
    }
}
```

---

## 개발 워크플로우

### 1. Phase 시작 전
- [ ] DEVELOPMENT_ROADMAP.md에서 해당 Phase 체크리스트 확인
- [ ] 필요한 dependencies 확인
- [ ] 관련 PRD 섹션 재확인

### 2. 개발 중
- [ ] 작은 단위로 커밋 (Incremental)
- [ ] 각 기능마다 테스트 작성
- [ ] `cargo check` / `cargo clippy` 자주 실행
- [ ] `cargo fmt` 실행

### 3. Phase 완료 후
- [ ] 전체 테스트 실행 (`cargo test`)
- [ ] 문서 업데이트
- [ ] DEVELOPMENT_ROADMAP.md 체크박스 업데이트
- [ ] Git commit with meaningful message

### 4. Git Commit Messages

```
feat: Add Task model with status tracking
fix: Resolve time conflict detection bug
refactor: Extract storage logic into trait
docs: Update API documentation for Schedule
test: Add integration tests for CLI commands
chore: Update dependencies
```

---

## Claude Code 연동 시 주의사항

### 1. Context 수집
- Git 정보는 신중하게 (민감 정보 체크)
- 파일 경로는 상대 경로 사용
- 큰 파일은 요약해서 전달

### 2. API 호출
- Rate limiting 고려
- 에러 핸들링 철저히
- Timeout 설정

### 3. 응답 파싱
- JSON 파싱 실패 대비
- Fallback 메시지 준비

---

## 유용한 Cargo 명령어

```bash
# 빠른 타입 체크
cargo check

# Clippy (린터)
cargo clippy

# 포맷팅
cargo fmt

# 테스트
cargo test

# 특정 테스트만
cargo test test_task_creation

# 문서 생성 및 열기
cargo doc --open

# 릴리즈 빌드
cargo build --release

# 의존성 트리 확인
cargo tree

# 사용하지 않는 의존성 찾기 (cargo-udeps 설치 필요)
cargo udeps
```

---

## 성능 프로파일링

```bash
# Criterion 벤치마크 (benches/ 디렉토리)
cargo bench

# Flamegraph (cargo-flamegraph 설치 필요)
cargo flamegraph

# Bloat 분석 (바이너리 크기)
cargo bloat --release
```

---

## 체크리스트: 코드 리뷰 전

- [ ] `cargo fmt` 실행됨
- [ ] `cargo clippy` 경고 없음
- [ ] 모든 테스트 통과 (`cargo test`)
- [ ] 새로운 public API는 문서화됨
- [ ] 에러 핸들링 적절함 (unwrap/panic 없음)
- [ ] 불필요한 clone/allocation 제거
- [ ] DEVELOPMENT_ROADMAP.md 업데이트

---

## 참고 자료

### Rust 공식
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### 이 프로젝트
- [PRD.md](./PRD.md) - 제품 요구사항
- [DEVELOPMENT_ROADMAP.md](./DEVELOPMENT_ROADMAP.md) - 개발 계획

### Crates
- [clap](https://docs.rs/clap/) - CLI
- [ratatui](https://docs.rs/ratatui/) - TUI
- [serde](https://docs.rs/serde/) - Serialization
- [chrono](https://docs.rs/chrono/) - Time handling
- [anyhow](https://docs.rs/anyhow/) - Error handling (간단한 용도)
- [thiserror](https://docs.rs/thiserror/) - Error types

---

**Last Updated**: 2025-10-30

**Note**: 이 문서는 프로젝트가 진행되면서 계속 업데이트됩니다.
