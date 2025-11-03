#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shift_schedule;
mod ai_provider;

use scheduler::{JsonStorage, Storage, Schedule, ScheduleChange, Task};
use chrono::{NaiveDate, Local, TimeZone, NaiveTime};
use serde::{Deserialize, Serialize};
use shift_schedule::shift_schedule;
use ai_provider::{AiProvider, AiConfig};

// Simple DTO for creating tasks from frontend
#[derive(Debug, Serialize, Deserialize)]
struct TaskInput {
    title: String,
    start_time: String,  // HH:MM format
    end_time: String,    // HH:MM format
    tags: Vec<String>,
    notes: Option<String>,
    pomodoro_duration: Option<u32>, // Optional: custom pomodoro duration in minutes
}

// Task suggestion from Claude
#[derive(Debug, Serialize, Deserialize)]
struct TaskSuggestion {
    suggested_title: String,
    suggested_start_time: String,
    suggested_end_time: String,
    tags: Vec<String>,
    notes: Option<String>,
    pomodoro_duration: u32,
    reasoning: String,
}

fn parse_time_on_date(date: NaiveDate, time_str: &str) -> Result<chrono::DateTime<Local>, String> {
    let time = NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|e| format!("Invalid time format: {}", e))?;
    let datetime = date.and_time(time);
    Local.from_local_datetime(&datetime)
        .single()
        .ok_or_else(|| "Invalid datetime".to_string())
}

// Get schedule for a specific date
#[tauri::command]
fn get_schedule(date: String) -> Result<Option<Schedule>, String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage.load_schedule(datetime).map_err(|e| e.to_string())?;

    // 통계 계산
    if let Some(ref mut sched) = schedule {
        sched.calculate_stats();
        // total_wasted는 현재 시간 기준이므로 매번 계산
        sched.total_wasted = Some(sched.total_wasted());
    }

    Ok(schedule)
}

// Get today's schedule
#[tauri::command]
fn get_today_schedule() -> Result<Option<Schedule>, String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let mut schedule = storage.load_today().map_err(|e| e.to_string())?;

    // 통계 계산
    if let Some(ref mut sched) = schedule {
        sched.calculate_stats();
        // total_wasted는 현재 시간 기준이므로 매번 계산
        sched.total_wasted = Some(sched.total_wasted());
    }

    Ok(schedule)
}

// Create a new schedule
#[tauri::command]
fn create_schedule(date: String, tasks: Vec<TaskInput>) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = Schedule::new(datetime);

    // Convert TaskInput to Task
    for task_input in tasks {
        let start = parse_time_on_date(parsed_date, &task_input.start_time)?;
        let end = parse_time_on_date(parsed_date, &task_input.end_time)?;
        let mut task = Task::new(task_input.title, start, end);
        task.tags = task_input.tags;
        task.notes = task_input.notes;
        task.custom_pomodoro_duration = task_input.pomodoro_duration;
        schedule.tasks.push(task);
    }

    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}

// Add a task to existing schedule
#[tauri::command]
fn add_task(date: String, task_input: TaskInput) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| Schedule::new(datetime));

    // Convert TaskInput to Task
    let start = parse_time_on_date(parsed_date, &task_input.start_time)?;
    let end = parse_time_on_date(parsed_date, &task_input.end_time)?;
    let mut task = Task::new(task_input.title, start, end);
    task.tags = task_input.tags;
    task.notes = task_input.notes;
    task.custom_pomodoro_duration = task_input.pomodoro_duration;

    schedule.tasks.push(task);
    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}

// Update a task - simplified version
#[tauri::command]
fn update_task(date: String, index: usize, task_input: TaskInput) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Schedule not found".to_string())?;

    if index >= schedule.tasks.len() {
        return Err("Task index out of bounds".to_string());
    }

    // Record old time for change history
    let old_start = schedule.tasks[index].start_time.format("%H:%M").to_string();
    let old_end = schedule.tasks[index].end_time.format("%H:%M").to_string();
    let old_time = format!("{}-{}", old_start, old_end);
    let task_title = schedule.tasks[index].title.clone();

    // Update task fields
    let start = parse_time_on_date(parsed_date, &task_input.start_time)?;
    let end = parse_time_on_date(parsed_date, &task_input.end_time)?;

    schedule.tasks[index].title = task_input.title.clone();
    schedule.tasks[index].start_time = start;
    schedule.tasks[index].end_time = end;
    schedule.tasks[index].tags = task_input.tags;
    schedule.tasks[index].notes = task_input.notes;

    // Record change if time changed
    let new_time = format!("{}-{}", task_input.start_time, task_input.end_time);
    if old_time != new_time {
        let change = ScheduleChange::task_updated(task_title, old_time, new_time);
        schedule.add_change(change);
    }

    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}

// Delete a task
#[tauri::command]
fn delete_task(date: String, index: usize) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Schedule not found".to_string())?;

    if index >= schedule.tasks.len() {
        return Err("Task index out of bounds".to_string());
    }

    schedule.tasks.remove(index);
    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}

// Start a task
#[tauri::command]
fn start_task(date: String, index: usize) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Schedule not found".to_string())?;

    if index >= schedule.tasks.len() {
        return Err("Task index out of bounds".to_string());
    }

    schedule.tasks[index].start();
    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}

// Pause a task
#[tauri::command]
fn pause_task(date: String, index: usize) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Schedule not found".to_string())?;

    if index >= schedule.tasks.len() {
        return Err("Task index out of bounds".to_string());
    }

    schedule.tasks[index].pause();
    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}

// Resume a task
#[tauri::command]
fn resume_task(date: String, index: usize) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Schedule not found".to_string())?;

    if index >= schedule.tasks.len() {
        return Err("Task index out of bounds".to_string());
    }

    schedule.tasks[index].resume();
    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}

// Complete a task with focus score
#[tauri::command]
fn complete_task(date: String, index: usize, _focus_score: u8) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Schedule not found".to_string())?;

    if index >= schedule.tasks.len() {
        return Err("Task index out of bounds".to_string());
    }

    // Use the complete() method from Task
    schedule.tasks[index].complete();

    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}

// Get weekly summary
#[tauri::command]
fn get_weekly_summary() -> Result<serde_json::Value, String> {
    // TODO: Implement weekly summary
    Ok(serde_json::json!({"message": "Weekly summary not yet implemented"}))
}

// Get monthly summary
#[tauri::command]
fn get_monthly_summary(year: i32, month: u32) -> Result<serde_json::Value, String> {
    // TODO: Implement monthly summary
    Ok(serde_json::json!({"year": year, "month": month, "message": "Monthly summary not yet implemented"}))
}

// Test command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Scheduler.", name)
}

// Check AI provider availability
#[tauri::command]
fn check_ai_provider(provider: String) -> Result<String, String> {
    let ai_provider = match provider.as_str() {
        "copilot" => AiProvider::Copilot,
        "claude" => AiProvider::Claude,
        other => return Err(format!("Unknown AI provider: {}", other)),
    };

    AiConfig::verify_cli(&ai_provider, None)
}

// Get AI provider installation guide
#[tauri::command]
fn get_ai_installation_guide(provider: String) -> Result<String, String> {
    let ai_provider = match provider.as_str() {
        "copilot" => AiProvider::Copilot,
        "claude" => AiProvider::Claude,
        other => return Err(format!("Unknown AI provider: {}", other)),
    };

    Ok(AiConfig::get_installation_guide(&ai_provider))
}

// AI Provider integration - Ask AI for advice (supports multiple providers)
#[tauri::command]
fn ask_ai(prompt: String, provider: Option<String>) -> Result<String, String> {
    let ai_provider = match provider.as_deref() {
        Some("copilot") => AiProvider::Copilot,
        Some("claude") | None => AiProvider::Claude, // Default to Claude
        Some(other) => return Err(format!("Unknown AI provider: {}", other)),
    };

    let config = AiConfig {
        provider: ai_provider,
        claude_path: None, // Use default paths (auto-detect)
        copilot_path: None,
    };

    config.ask(&prompt)
}

// Claude Code integration - Ask Claude for advice (deprecated, use ask_ai instead)
// Kept for backwards compatibility
#[tauri::command]
fn ask_claude(prompt: String) -> Result<String, String> {
    ask_ai(prompt, Some("claude".to_string()))
}

// Evaluate today's schedule with Claude
#[tauri::command]
fn evaluate_schedule(date: String) -> Result<String, String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No schedule found for this date".to_string())?;

    if schedule.tasks.is_empty() {
        return Err("No tasks in schedule".to_string());
    }

    // Build schedule description for Claude
    let mut prompt = format!("오늘({}) 내 스케줄:\n", date);
    for (i, task) in schedule.tasks.iter().enumerate() {
        let start = task.start_time.format("%H:%M").to_string();
        let end = task.end_time.format("%H:%M").to_string();
        prompt.push_str(&format!(
            "{}. {} ({}-{}, {}분)\n",
            i + 1,
            task.title,
            start,
            end,
            task.estimated_duration_minutes
        ));
    }

    prompt.push_str("\n이 스케줄을 15가지 심리학 원리와 시간 관리 방법론에 기반하여 평가하고, 개선점을 제시해주세요:\n\n");

    prompt.push_str("## 평가 기준 (적용된 심리학 원리):\n\n");
    prompt.push_str("1. **파킨슨의 법칙 (Parkinson's Law)**: 각 작업의 예상 시간이 너무 여유롭지 않은가? 작업은 주어진 시간만큼 확장되므로, 적절히 타이트한 데드라인이 효율성을 높입니다.\n");
    prompt.push_str("2. **자이가르닉 효과 (Zeigarnik Effect)**: 미완료 작업을 명확히 인식할 수 있도록 작업이 구체적으로 정의되어 있는가? 불완전한 작업은 완료하려는 동기를 유발합니다.\n");
    prompt.push_str("3. **희소성 원리 (Scarcity Principle)**: 하루 24시간이라는 한정된 자원을 고려했을 때, 시간 배분이 적절한가?\n");
    prompt.push_str("4. **포모도로 기법 (Pomodoro)**: 작업 시간이 25-90분 단위로 적절히 분할되어 있는가? 너무 긴 작업은 집중력 저하를 초래합니다.\n");
    prompt.push_str("5. **몰입 상태 (Flow State)**: 작업 난이도가 너무 쉽거나 어렵지 않은가? 적절한 도전 과제가 몰입을 유도합니다.\n");
    prompt.push_str("6. **에너지 관리 (Energy Management)**: 휴식 시간이 적절히 배치되어 있는가? 지속 가능한 생산성을 위해 에너지 회복이 필요합니다.\n");
    prompt.push_str("7. **타임 블로킹 (Time Blocking)**: 작업 간 전환 시간(5-10분)이 충분한가?\n");
    prompt.push_str("8. **손실 회피 (Loss Aversion)**: 시간 낭비를 최소화하는 구조인가?\n\n");

    prompt.push_str("## 구체적으로 평가해주세요:\n");
    prompt.push_str("1. 각 작업의 예상 시간이 현실적인가? (파킨슨의 법칙 고려)\n");
    prompt.push_str("2. 작업이 구체적이고 명확하게 정의되어 있는가? (자이가르닉 효과)\n");
    prompt.push_str("3. 작업 간 전환 시간이 충분한가?\n");
    prompt.push_str("4. 휴식 시간이 적절한가? (에너지 관리)\n");
    prompt.push_str("5. 집중력 리듬을 고려했을 때 작업 순서가 적절한가?\n");
    prompt.push_str("6. 개선이 필요한 작업이 있다면 구체적으로 제시해주세요.\n\n");
    prompt.push_str("답변은 마크다운 형식으로 작성해주세요.");

    ask_claude(prompt)
}

// Send notification
#[tauri::command]
fn send_notification(title: String, body: String) -> Result<(), String> {
    use tauri::api::notification::Notification;

    Notification::new("com.scheduler.app")
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| format!("Failed to send notification: {}", e))
}

// Get advice for a new task (supports multiple AI providers)
#[tauri::command]
fn get_task_advice(title: String, duration_minutes: i64, provider: Option<String>) -> Result<String, String> {
    let prompt = format!(
        "작업 제목: \"{}\"\n예상 소요 시간: {}분\n\n\
        다음 심리학 원리와 시간 관리 방법론을 기반으로 조언해주세요:\n\n\
        ## 적용할 원리:\n\
        1. **파킨슨의 법칙**: 이 예상 시간이 너무 여유롭지 않나요? 작업은 주어진 시간만큼 확장되므로, 적절히 타이트한 데드라인을 권장합니다.\n\
        2. **자이가르닉 효과**: 작업을 명확하고 구체적으로 정의하면 완료하려는 동기가 높아집니다. 이 작업명이 충분히 구체적인가요?\n\
        3. **포모도로 기법**: 최적의 집중 세션 시간을 추천해주세요 (15, 25, 45, 60, 90분 중).\n\
        4. **몰입 상태**: 이 작업의 난이도와 복잡도를 고려한 접근 방법을 제시해주세요.\n\n\
        ## 구체적으로 답변해주세요:\n\
        1. 예상 시간 {}분이 현실적인가? (파킨슨의 법칙 고려)\n\
        2. 작업을 더 구체적으로 정의하려면? (자이가르닉 효과)\n\
        3. 추천 Pomodoro 세션 시간과 그 이유는?\n\
        4. 효율적 수행을 위한 핵심 팁 1-2가지\n\
        5. 주의할 점이나 흔한 실수\n\n\
        간결하게 5-7문장으로 답변해주세요.",
        title, duration_minutes, duration_minutes
    );

    ask_ai(prompt, provider)
}

// Auto-complete task creation with AI (accepts natural language, supports multiple providers)
#[tauri::command]
fn suggest_task_completion(date: String, user_input: String, provider: Option<String>) -> Result<TaskSuggestion, String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let schedule = storage.load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| Schedule::new(datetime));

    // Build existing schedule description
    let mut schedule_desc = String::new();
    if schedule.tasks.is_empty() {
        schedule_desc.push_str("오늘은 아직 스케줄이 비어있습니다.\n");
    } else {
        schedule_desc.push_str("오늘 기존 스케줄:\n");
        for (i, task) in schedule.tasks.iter().enumerate() {
            let start = task.start_time.format("%H:%M").to_string();
            let end = task.end_time.format("%H:%M").to_string();
            schedule_desc.push_str(&format!(
                "{}. {} ({}-{}, {}분)\n",
                i + 1,
                task.title,
                start,
                end,
                task.estimated_duration_minutes
            ));
        }
    }

    let prompt = format!(
        "{}\n\
        사용자 요청: \"{}\"\n\n\
        IMPORTANT: You must respond with ONLY valid JSON. No explanation, no markdown, just raw JSON.\n\n\
        ## 심리학 원리 기반 작업 생성:\n\
        다음 원리들을 고려하여 작업을 제안하세요:\n\
        1. **자이가르닉 효과**: 작업 제목을 구체적이고 명확하게 정의하세요. 모호한 작업명은 완료 동기를 낮춥니다.\n\
        2. **파킨슨의 법칙**: 예상 시간을 적절히 타이트하게 설정하세요. 너무 여유로운 시간은 비효율을 초래합니다.\n\
        3. **포모도로 기법**: 작업을 25-90분 단위로 분할 가능하도록 pomodoro_duration을 선택하세요.\n\
        4. **희소성 원리**: 하루 24시간이라는 한정된 자원을 고려하여 시간을 배분하세요.\n\n\
        Output JSON structure:\n\
        {{\n\
          \"suggested_title\": \"task title (구체적이고 명확하게)\",\n\
          \"suggested_start_time\": \"HH:MM\",\n\
          \"suggested_end_time\": \"HH:MM\",\n\
          \"tags\": [\"tag1\", \"tag2\"],\n\
          \"notes\": \"tips or notes for this task\",\n\
          \"pomodoro_duration\": 25,\n\
          \"reasoning\": \"why you made these suggestions (심리학 원리 언급)\"\n\
        }}\n\n\
        Rules:\n\
        1. suggested_title: 구체적으로 추출 (e.g., \"조깅 30분\" → \"30분 조깅 (XX 코스)\")\n\
        2. suggested_start_time and suggested_end_time: 기존 스케줄과 충돌 회피\n\
           - \"아침\" → 07:00-09:00\n\
           - \"오전\" → 09:00-12:00\n\
           - \"오후\" → 13:00-18:00\n\
           - \"저녁\" → 18:00-21:00\n\
        3. Duration: 지정된 경우 그대로, 아니면 파킨슨 법칙에 따라 타이트하게\n\
        4. tags: 1-3개 (운동, 업무, 학습, 건강, 휴식 등)\n\
        5. notes: 실행 팁 1-2문장 (자이가르닉 효과 활용 - 구체적 실행 방법 제시)\n\
        6. pomodoro_duration: [5, 15, 25, 45, 60, 90] 중 작업 특성에 맞게 선택\n\
        7. reasoning: 심리학 원리를 언급하며 1문장으로 설명\n\n\
        Output ONLY the JSON object, nothing else:",
        schedule_desc, user_input
    );

    let response = ask_ai(prompt, provider)?;

    // Log response for debugging
    eprintln!("AI Response: {}", response);

    // Try to extract JSON from response (handles both clean JSON and wrapped responses)
    let json_str = if let Some(start) = response.find('{') {
        if let Some(end) = response[start..].rfind('}') {
            &response[start..start + end + 1]
        } else {
            return Err(format!("Incomplete JSON in response: {}", response));
        }
    } else {
        return Err(format!("No JSON object found in response: {}", response));
    };

    serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse JSON: {}. Extracted: {}", e, json_str))
}

fn main() {
    use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent};
    use tauri::Manager;

    // Create system tray menu
    let show = CustomMenuItem::new("show".to_string(), "Show Window");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide Window");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                // Show window on tray icon click
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "show" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "hide" => {
                        let window = app.get_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_schedule,
            get_today_schedule,
            create_schedule,
            add_task,
            update_task,
            delete_task,
            start_task,
            pause_task,
            resume_task,
            complete_task,
            get_weekly_summary,
            get_monthly_summary,
            check_ai_provider,
            get_ai_installation_guide,
            ask_ai,
            ask_claude,
            evaluate_schedule,
            get_task_advice,
            suggest_task_completion,
            send_notification,
            shift_schedule,
            greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
