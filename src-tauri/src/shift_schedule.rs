// Shift schedule command - extract for change history tracking
use scheduler::{JsonStorage, Storage, ScheduleChange};
use chrono::{NaiveDate, Local, TimeZone};

#[tauri::command]
pub fn shift_schedule(
    date: String,
    from_index: usize,
    shift_minutes: i64,
) -> Result<(), String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let datetime = Local.from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("Invalid datetime".to_string())?;

    let mut schedule = storage
        .load_schedule(datetime)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Schedule not found".to_string())?;

    if from_index >= schedule.tasks.len() {
        return Err("Task index out of bounds".to_string());
    }

    let from_task_title = schedule.tasks[from_index].title.clone();
    let affected_count = schedule.tasks.len() - from_index;

    // Shift all tasks from from_index onwards
    for i in from_index..schedule.tasks.len() {
        let task = &mut schedule.tasks[i];
        let mut start_time = task.start_time.naive_local();
        let mut end_time = task.end_time.naive_local();

        start_time = start_time + chrono::Duration::minutes(shift_minutes);
        end_time = end_time + chrono::Duration::minutes(shift_minutes);

        task.start_time = Local.from_local_datetime(&start_time)
            .single()
            .ok_or("Invalid datetime after shift".to_string())?;
        task.end_time = Local.from_local_datetime(&end_time)
            .single()
            .ok_or("Invalid datetime after shift".to_string())?;
    }

    // Record change history
    let change = ScheduleChange::schedule_shifted(from_task_title, shift_minutes, affected_count);
    schedule.add_change(change);

    storage.save_schedule(&schedule).map_err(|e| e.to_string())
}
