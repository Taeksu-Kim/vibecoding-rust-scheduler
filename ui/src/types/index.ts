// Types matching the Rust backend structures

export type TaskStatus = 'Pending' | 'InProgress' | 'Completed' | 'Paused' | 'Skipped';

export type ChangeType =
  | 'TaskCreated'
  | 'TaskUpdated'
  | 'TaskDeleted'
  | 'TaskMoved'
  | 'ScheduleShifted';

export interface ScheduleChange {
  timestamp: string; // ISO datetime string
  change_type: ChangeType;
  task_title?: string;
  old_time?: string; // HH:MM-HH:MM format
  new_time?: string; // HH:MM-HH:MM format
  affected_tasks_count?: number;
  description: string;
}

export interface PomodoroSession {
  total_pomodoros: number;
  completed_pomodoros: number;
  current_start?: string; // ISO datetime string
  pomodoro_duration: number;
  short_break: number;
  long_break: number;
}

// Full Task from backend (read-only)
export interface Task {
  id: string;
  title: string;
  start_time: string; // ISO datetime string
  end_time: string;   // ISO datetime string
  estimated_duration_minutes: number;
  actual_duration_minutes?: number;
  status: TaskStatus;
  tags: string[];
  notes?: string;
  actual_start_time?: string;
  actual_end_time?: string;
  pomodoro?: PomodoroSession;
}

// Simple TaskInput for creating/updating tasks
export interface TaskInput {
  title: string;
  start_time: string; // HH:MM format
  end_time: string;   // HH:MM format
  tags: string[];
  notes?: string;
  pomodoro_duration?: number; // Optional: 5, 10, 15, 20, 25, 30, 45, 60, 90 minutes
}

export interface Schedule {
  date: string; // ISO datetime string
  tasks: Task[];
  changes: ScheduleChange[]; // Schedule change history
  completion_rate?: number;
  efficiency_score?: number;
  total_earned?: number;
  total_wasted?: number;
  total_bonus?: number;
  total_penalty?: number;
}

export interface DailySummary {
  date: string;
  efficiency_score: number;
  grade: string;
  net_earned: number;
  total_wasted: number;
}

export interface TaskSuggestion {
  suggested_title: string;
  suggested_start_time: string;
  suggested_end_time: string;
  tags: string[];
  notes?: string;
  pomodoro_duration: number;
  reasoning: string;
}

export type ViewMode = 'schedule' | 'timeline' | 'dashboard';
