// Tauri IPC API service
import { invoke } from '@tauri-apps/api/tauri';
import type { Schedule, TaskInput, TaskSuggestion } from '../types';

export const api = {
  // Get schedule for a specific date
  async getSchedule(date: string): Promise<Schedule | null> {
    return await invoke('get_schedule', { date });
  },

  // Get today's schedule
  async getTodaySchedule(): Promise<Schedule | null> {
    return await invoke('get_today_schedule');
  },

  // Create a new schedule
  async createSchedule(date: string, tasks: TaskInput[]): Promise<void> {
    return await invoke('create_schedule', { date, tasks });
  },

  // Add a task to existing schedule
  async addTask(date: string, taskInput: TaskInput): Promise<void> {
    return await invoke('add_task', { date, taskInput });
  },

  // Update a task
  async updateTask(date: string, index: number, taskInput: TaskInput): Promise<void> {
    return await invoke('update_task', { date, index, taskInput });
  },

  // Delete a task
  async deleteTask(date: string, index: number): Promise<void> {
    return await invoke('delete_task', { date, index });
  },

  // Start a task
  async startTask(date: string, index: number): Promise<void> {
    return await invoke('start_task', { date, index });
  },

  // Pause a task
  async pauseTask(date: string, index: number): Promise<void> {
    return await invoke('pause_task', { date, index });
  },

  // Resume a task
  async resumeTask(date: string, index: number): Promise<void> {
    return await invoke('resume_task', { date, index });
  },

  // Mark task as completed
  async completeTask(date: string, index: number, focusScore: number): Promise<void> {
    return await invoke('complete_task', { date, index, focusScore });
  },

  // Get weekly summary
  async getWeeklySummary(): Promise<any> {
    return await invoke('get_weekly_summary');
  },

  // Get monthly summary
  async getMonthlySummary(year: number, month: number): Promise<any> {
    return await invoke('get_monthly_summary', { year, month });
  },

  // Check if AI provider is available
  async checkAIProvider(provider: string): Promise<string> {
    return await invoke('check_ai_provider', { provider });
  },

  // Get AI provider installation guide
  async getAIInstallationGuide(provider: string): Promise<string> {
    return await invoke('get_ai_installation_guide', { provider });
  },

  // Ask AI for advice (supports multiple providers: claude, copilot)
  async askAI(prompt: string, provider?: string): Promise<string> {
    return await invoke('ask_ai', { prompt, provider });
  },

  // Ask Claude for advice (deprecated, use askAI instead)
  async askClaude(prompt: string): Promise<string> {
    return await invoke('ask_claude', { prompt });
  },

  // Evaluate schedule with Claude
  async evaluateSchedule(date: string): Promise<string> {
    return await invoke('evaluate_schedule', { date });
  },

  // Get task advice from AI (supports multiple providers)
  async getTaskAdvice(title: string, durationMinutes: number, provider?: string): Promise<string> {
    return await invoke('get_task_advice', { title, durationMinutes, provider });
  },

  // Auto-complete task details (accepts natural language, supports multiple providers)
  async suggestTaskCompletion(date: string, userInput: string, provider?: string): Promise<TaskSuggestion> {
    return await invoke('suggest_task_completion', { date, userInput, provider });
  },

  // Send notification
  async sendNotification(title: string, body: string): Promise<void> {
    return await invoke('send_notification', { title, body });
  },

  // Shift schedule (postpone/pull forward)
  async shiftSchedule(date: string, fromIndex: number, shiftMinutes: number): Promise<void> {
    return await invoke('shift_schedule', { date, fromIndex, shiftMinutes });
  },
};
