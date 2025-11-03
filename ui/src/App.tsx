import { useState, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { ScheduleView } from './components/ScheduleView';
import { TimelineView } from './components/TimelineView';
import { DashboardView } from './components/DashboardView';
import { api } from './services/api';
import { notificationService } from './services/notificationService';
import type { Schedule, TaskInput, ViewMode } from './types';
import './App.css';

function App() {
  const [currentView, setCurrentView] = useState<ViewMode>('schedule');
  const [selectedDate, setSelectedDate] = useState(() => {
    const today = new Date();
    const year = today.getFullYear();
    const month = String(today.getMonth() + 1).padStart(2, '0');
    const day = String(today.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  });
  const [schedule, setSchedule] = useState<Schedule | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadSchedule = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.getSchedule(selectedDate);
      setSchedule(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load schedule');
      console.error('Failed to load schedule:', err);
    } finally {
      setLoading(false);
    }
  };

  // Load schedule when date changes
  useEffect(() => {
    loadSchedule();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedDate]);

  // Start notification service when schedule changes
  useEffect(() => {
    notificationService.reset();
    notificationService.start(schedule, selectedDate);

    return () => {
      notificationService.stop();
    };
  }, [schedule, selectedDate]);

  const handleAddTask = async (taskInput: TaskInput) => {
    try {
      if (schedule) {
        // Add to existing schedule
        await api.addTask(selectedDate, taskInput);
      } else {
        // Create new schedule with this task
        await api.createSchedule(selectedDate, [taskInput]);
      }
      await loadSchedule();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add task');
      console.error('Failed to add task:', err);
    }
  };

  const handleUpdateTask = async (index: number, taskInput: TaskInput) => {
    try {
      await api.updateTask(selectedDate, index, taskInput);
      await loadSchedule();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update task');
      console.error('Failed to update task:', err);
    }
  };

  const handleDeleteTask = async (index: number) => {
    try {
      await api.deleteTask(selectedDate, index);
      await loadSchedule();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete task');
      console.error('Failed to delete task:', err);
    }
  };

  const handleStartTask = async (index: number) => {
    try {
      await api.startTask(selectedDate, index);
      await loadSchedule();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start task');
      console.error('Failed to start task:', err);
    }
  };

  const handlePauseTask = async (index: number) => {
    try {
      await api.pauseTask(selectedDate, index);
      await loadSchedule();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to pause task');
      console.error('Failed to pause task:', err);
    }
  };

  const handleResumeTask = async (index: number) => {
    try {
      await api.resumeTask(selectedDate, index);
      await loadSchedule();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to resume task');
      console.error('Failed to resume task:', err);
    }
  };

  const handleCompleteTask = async (index: number, focusScore: number) => {
    try {
      await api.completeTask(selectedDate, index, focusScore);
      await loadSchedule();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to complete task');
      console.error('Failed to complete task:', err);
    }
  };

  const handleShiftSchedule = async (fromIndex: number, shiftMinutes: number) => {
    if (!schedule) return;

    try {
      // Call shift_schedule API (handles change history tracking)
      await api.shiftSchedule(selectedDate, fromIndex, shiftMinutes);

      // Reload schedule to get updated data with change history
      await loadSchedule();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to shift schedule');
      console.error('Failed to shift schedule:', err);
    }
  };

  return (
    <div className="flex h-screen bg-gray-50">
      <Sidebar
        currentView={currentView}
        onViewChange={setCurrentView}
        selectedDate={selectedDate}
        onDateChange={setSelectedDate}
      />

      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Error Toast */}
        {error && (
          <div className="bg-red-50 border-l-4 border-red-500 p-4 m-4">
            <div className="flex items-center">
              <span className="text-red-700">{error}</span>
              <button
                onClick={() => setError(null)}
                className="ml-auto text-red-700 hover:text-red-900"
              >
                ✕
              </button>
            </div>
          </div>
        )}

        {/* Loading Indicator */}
        {loading && (
          <div className="bg-blue-50 border-l-4 border-blue-500 p-4 m-4">
            <span className="text-blue-700">Loading...</span>
          </div>
        )}

        {/* Main Content */}
        {currentView === 'schedule' && (
          <ScheduleView
            schedule={schedule}
            selectedDate={selectedDate}
            onAddTask={handleAddTask}
            onUpdateTask={handleUpdateTask}
            onDeleteTask={handleDeleteTask}
            onStartTask={handleStartTask}
            onPauseTask={handlePauseTask}
            onResumeTask={handleResumeTask}
            onCompleteTask={handleCompleteTask}
          />
        )}

        {currentView === 'timeline' && (
          <TimelineView
            schedule={schedule}
            selectedDate={selectedDate}
            onUpdateTask={handleUpdateTask}
            onShiftSchedule={handleShiftSchedule}
          />
        )}

        {currentView === 'dashboard' && (
          <DashboardView schedule={schedule} selectedDate={selectedDate} />
        )}
      </div>
    </div>
  );
}

export default App;
