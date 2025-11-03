import { api } from './api';
import type { Schedule } from '../types';

export class NotificationService {
  private checkInterval: number | null = null;
  private notifiedTasks: Set<string> = new Set();
  private notifiedEnds: Set<string> = new Set();

  start(schedule: Schedule | null, selectedDate: string) {
    // Clear existing interval
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
    }

    // Check every 30 seconds
    this.checkInterval = window.setInterval(() => {
      if (schedule) {
        this.checkNotifications(schedule, selectedDate);
      }
    }, 30000); // 30 seconds

    // Also check immediately
    if (schedule) {
      this.checkNotifications(schedule, selectedDate);
    }
  }

  stop() {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = null;
    }
  }

  private checkNotifications(schedule: Schedule, selectedDate: string) {
    const now = new Date();
    const today = new Date();
    const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;

    // Only send notifications for today's schedule
    if (selectedDate !== todayStr) {
      return;
    }

    schedule.tasks.forEach((task) => {
      const taskStart = new Date(task.start_time);
      const taskEnd = new Date(task.end_time);
      const taskId = `${task.title}-${task.start_time}`;

      // Calculate time until start (in minutes)
      const minutesUntilStart = (taskStart.getTime() - now.getTime()) / (1000 * 60);

      // Notify 5 minutes before task starts
      if (minutesUntilStart > 0 && minutesUntilStart <= 5 && !this.notifiedTasks.has(taskId)) {
        this.notifiedTasks.add(taskId);
        api.sendNotification(
          '⏰ 작업 시작 예정',
          `"${task.title}" 작업이 ${Math.ceil(minutesUntilStart)}분 후에 시작됩니다.`
        ).catch(console.error);
      }

      // Notify when task should end
      const minutesUntilEnd = (taskEnd.getTime() - now.getTime()) / (1000 * 60);
      if (minutesUntilEnd > -1 && minutesUntilEnd <= 1 && !this.notifiedEnds.has(taskId)) {
        this.notifiedEnds.add(taskId);
        api.sendNotification(
          '✅ 작업 종료 시간',
          `"${task.title}" 작업 종료 시간입니다.`
        ).catch(console.error);
      }
    });
  }

  // Reset notifications (call this when date changes or schedule reloads)
  reset() {
    this.notifiedTasks.clear();
    this.notifiedEnds.clear();
  }
}

// Singleton instance
export const notificationService = new NotificationService();
