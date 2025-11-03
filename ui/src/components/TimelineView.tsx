import { useState, useEffect } from 'react';
import type { Schedule, Task, TaskInput, TaskStatus } from '../types';

interface TimelineViewProps {
  schedule: Schedule | null;
  selectedDate: string;
  onUpdateTask?: (index: number, taskInput: TaskInput) => void;
  onShiftSchedule?: (fromIndex: number, shiftMinutes: number) => void;
}

export function TimelineView({ schedule, selectedDate, onUpdateTask, onShiftSchedule }: TimelineViewProps) {
  const [currentTime, setCurrentTime] = useState(new Date());
  const [isDragging, setIsDragging] = useState(false);
  const [draggedTask, setDraggedTask] = useState<{ task: Task; index: number } | null>(null);
  const [dragOverHour, setDragOverHour] = useState<number | null>(null);
  const [shiftModalOpen, setShiftModalOpen] = useState(false);
  const [shiftDirection, setShiftDirection] = useState<'forward' | 'backward'>('forward');
  const [shiftTaskIndex, setShiftTaskIndex] = useState<number>(0);
  const [includeCurrentTask, setIncludeCurrentTask] = useState(true);
  const [shiftMinutes, setShiftMinutes] = useState(15);
  const hours = Array.from({ length: 24 }, (_, i) => i);

  // Debug: Log schedule when it changes
  useEffect(() => {
    console.log('Timeline: schedule updated', {
      hasSchedule: !!schedule,
      taskCount: schedule?.tasks.length || 0,
      selectedDate,
      tasks: schedule?.tasks.map(t => ({
        title: t.title,
        start: t.start_time,
        end: t.end_time
      }))
    });
  }, [schedule, selectedDate]);

  // Check if selected date is today (use local date, not UTC)
  const getTodayString = () => {
    const today = new Date();
    const year = today.getFullYear();
    const month = String(today.getMonth() + 1).padStart(2, '0');
    const day = String(today.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  };
  const isToday = selectedDate === getTodayString();

  // 희소성 원리: 남은 생산적인 시간 계산
  const calculateRemainingProductiveTime = () => {
    if (!schedule || !isToday) return null;

    const now = new Date();
    const currentHour = now.getHours();
    const currentMinute = now.getMinutes();
    const currentTotalMinutes = currentHour * 60 + currentMinute;

    // 남은 작업 중에서 완료되지 않은 작업들의 총 시간
    const remainingTasks = schedule.tasks.filter(task => {
      const taskEnd = new Date(task.end_time);
      return task.status !== 'Completed' && taskEnd > now;
    });

    const totalRemainingMinutes = remainingTasks.reduce((sum, task) => {
      return sum + task.estimated_duration_minutes;
    }, 0);

    // 오늘 하루 남은 시간 (23:59까지)
    const endOfDay = 24 * 60 - 1; // 23:59
    const remainingDayMinutes = endOfDay - currentTotalMinutes;

    return {
      totalRemainingMinutes,
      remainingDayMinutes,
      remainingTaskCount: remainingTasks.length,
    };
  };

  const remainingTime = calculateRemainingProductiveTime();

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  // Mouse event listeners for drag & drop
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging) return;

      // Calculate which hour slot we're over
      const targetHour = calculateTargetHour(e.clientY);
      setDragOverHour(targetHour);
    };

    const handleMouseUp = (e: MouseEvent) => {
      if (!isDragging || !draggedTask) return;

      console.log('🖱️ MouseUp at', e.clientX, e.clientY);

      // Find target hour
      const targetHour = calculateTargetHour(e.clientY);

      if (targetHour !== null) {
        console.log('📍 Dropping at hour:', targetHour);
        updateTaskTime(targetHour);
      }

      // Reset drag state
      setIsDragging(false);
      setDraggedTask(null);
      setDragOverHour(null);
    };

    if (isDragging) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);

      return () => {
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isDragging, draggedTask]);

  // Calculate current time position (0-24 hours in decimal)
  const currentTimeDecimal = isToday
    ? currentTime.getHours() + currentTime.getMinutes() / 60 + currentTime.getSeconds() / 3600
    : -1;

  const getTasksForHour = (hour: number): Task[] => {
    if (!schedule) return [];

    const tasksForHour = schedule.tasks.filter((task) => {
      const startDate = new Date(task.start_time);
      const endDate = new Date(task.end_time);

      const startHour = startDate.getHours();
      const endHour = endDate.getHours();

      // Task overlaps with this hour if:
      // - Single-hour task: startHour === endHour === hour
      // - Multi-hour task: hour is between startHour and endHour (inclusive)
      if (startHour === endHour) {
        // Single-hour task (e.g., 14:00-14:30)
        return hour === startHour;
      } else {
        // Multi-hour task (e.g., 14:00-16:00)
        return hour >= startHour && hour <= endHour;
      }
    });

    // Debug logging
    if (tasksForHour.length > 0) {
      console.log(`Hour ${hour}: Found ${tasksForHour.length} tasks`, tasksForHour.map(t => t.title));
    }

    return tasksForHour;
  };

  const formatTime = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
  };

  const getTaskColor = (task: Task) => {
    const statusColors: Record<TaskStatus, string> = {
      Completed: 'bg-green-500 border-green-600',
      InProgress: 'bg-orange-500 border-orange-600',
      Paused: 'bg-yellow-500 border-yellow-600',
      Pending: 'bg-gray-400 border-gray-500',
      Skipped: 'bg-red-400 border-red-500',
    };
    return statusColors[task.status] || 'bg-gray-400 border-gray-500';
  };

  const getTaskIcon = (task: Task) => {
    const icons: Record<TaskStatus, string> = {
      Completed: '✓',
      InProgress: '▶',
      Paused: '⏸',
      Pending: '○',
      Skipped: '✕',
    };
    return icons[task.status] || '○';
  };

  // Mouse-based drag handlers
  const handleMouseDown = (e: React.MouseEvent, task: Task, index: number) => {
    if (!onUpdateTask) return;

    console.log('🖱️ MouseDown:', task.title, 'index:', index);
    e.preventDefault();
    e.stopPropagation();

    setIsDragging(true);
    setDraggedTask({ task, index });
  };

  const calculateTargetHour = (clientY: number): number | null => {
    // Find the hour slot under the mouse
    const elements = document.elementsFromPoint(window.innerWidth / 2, clientY);

    for (const el of elements) {
      const hourAttr = el.getAttribute('data-hour');
      if (hourAttr !== null) {
        return parseInt(hourAttr, 10);
      }
    }

    return null;
  };

  const updateTaskTime = (targetHour: number) => {
    if (!draggedTask || !onUpdateTask || !schedule) return;

    const { task, index } = draggedTask;

    // Calculate duration
    const oldStart = new Date(task.start_time);
    const oldEnd = new Date(task.end_time);
    const durationMs = oldEnd.getTime() - oldStart.getTime();

    // Create new start/end times
    const [year, month, day] = selectedDate.split('-').map(Number);
    const newStart = new Date(year, month - 1, day, targetHour, 0, 0);
    const newEnd = new Date(newStart.getTime() + durationMs);

    // Format times as HH:MM
    const formatTime = (date: Date) => {
      const h = String(date.getHours()).padStart(2, '0');
      const m = String(date.getMinutes()).padStart(2, '0');
      return `${h}:${m}`;
    };

    // Update task
    const updatedTask: TaskInput = {
      title: task.title,
      start_time: formatTime(newStart),
      end_time: formatTime(newEnd),
      tags: task.tags,
      notes: task.notes,
      pomodoro_duration: task.pomodoro?.pomodoro_duration,
    };

    console.log('✅ Updating task to hour', targetHour, ':', updatedTask);
    onUpdateTask(index, updatedTask);
  };

  // Shift schedule handlers
  const openShiftModal = (taskIndex: number, direction: 'forward' | 'backward') => {
    setShiftTaskIndex(taskIndex);
    setShiftDirection(direction);
    setIncludeCurrentTask(true);
    setShiftMinutes(15);
    setShiftModalOpen(true);
  };

  const handleConfirmShift = () => {
    if (!onShiftSchedule) return;

    const startIndex = includeCurrentTask ? shiftTaskIndex : shiftTaskIndex + 1;
    const minutes = shiftDirection === 'forward' ? shiftMinutes : -shiftMinutes;

    console.log(`${shiftDirection === 'forward' ? '⏩' : '⏪'} Shifting from task ${startIndex} by ${minutes} minutes`);
    onShiftSchedule(startIndex, minutes);

    setShiftModalOpen(false);
  };

  return (
    <div className="flex-1 bg-gray-50 overflow-auto">
      <div className="max-w-6xl mx-auto p-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-4xl font-bold text-gray-800 mb-2">⏰ Timeline</h1>
          <p className="text-gray-600">
            {new Date(selectedDate).toLocaleDateString('ko-KR', {
              weekday: 'long',
              year: 'numeric',
              month: 'long',
              day: 'numeric',
            })}
          </p>
          {isToday && (
            <p className="text-sm text-gray-500 mt-1">
              현재 시간: {currentTime.toLocaleTimeString('ko-KR')}
            </p>
          )}
        </div>

        {/* 희소성 원리: 남은 생산적인 시간 표시 */}
        {remainingTime && (
          <div className="mb-6 bg-gradient-to-r from-orange-50 to-red-50 border-2 border-orange-300 rounded-lg shadow-md p-5">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-3">
                <span className="text-3xl">⏳</span>
                <div>
                  <h3 className="text-lg font-bold text-gray-800">오늘 남은 생산적인 시간</h3>
                  <p className="text-sm text-gray-600 mt-1">
                    시간은 유한합니다. 지금 이 순간을 놓치지 마세요!
                  </p>
                </div>
              </div>
              <div className="text-right">
                <div className="text-4xl font-bold text-orange-600">
                  {Math.floor(remainingTime.totalRemainingMinutes / 60)}시간 {remainingTime.totalRemainingMinutes % 60}분
                </div>
                <div className="text-sm text-gray-600 mt-1">
                  남은 작업: {remainingTime.remainingTaskCount}개
                </div>
                <div className="text-xs text-gray-500 mt-1">
                  (하루 남은 시간: {Math.floor(remainingTime.remainingDayMinutes / 60)}시간 {remainingTime.remainingDayMinutes % 60}분)
                </div>
              </div>
            </div>
            <div className="mt-3 bg-white bg-opacity-60 rounded-full h-3 overflow-hidden">
              <div
                className="bg-gradient-to-r from-orange-400 to-red-500 h-3 transition-all duration-500"
                style={{
                  width: `${Math.min((remainingTime.totalRemainingMinutes / remainingTime.remainingDayMinutes) * 100, 100)}%`
                }}
              />
            </div>
            <p className="text-xs text-gray-600 mt-2 text-center">
              💡 <strong>희소성 원리:</strong> 한정된 시간이라는 인식이 집중력과 동기를 높입니다
            </p>
          </div>
        )}

        {schedule ? (
          <div className="bg-white rounded-lg shadow-md p-6 relative">
            <div className="space-y-0">
              {hours.map((hour) => {
                const tasks = getTasksForHour(hour);
                const hasTask = tasks.length > 0;

                return (
                  <div key={hour} className="flex items-stretch min-h-[80px] relative">
                    {/* Hour Label */}
                    <div className="w-20 flex items-start justify-end pr-4 pt-1 text-sm font-semibold text-gray-600">
                      {hour.toString().padStart(2, '0')}:00
                    </div>

                    {/* Timeline Bar */}
                    <div
                      data-hour={hour}
                      className={`flex-1 border-l-2 border-gray-300 pl-4 relative ${
                        dragOverHour === hour ? 'bg-blue-50' : ''
                      }`}
                    >
                      {/* Current Time Indicator */}
                      {isToday && currentTimeDecimal >= hour && currentTimeDecimal < hour + 1 && (
                        <div
                          className="absolute left-0 right-0 z-20 flex items-center"
                          style={{
                            top: `${((currentTimeDecimal - hour) * 80)}px`,
                          }}
                        >
                          <div className="w-3 h-3 bg-red-500 rounded-full -ml-[7px]"></div>
                          <div className="flex-1 h-0.5 bg-red-500"></div>
                          <div className="bg-red-500 text-white text-xs px-2 py-0.5 rounded ml-2 font-semibold">
                            NOW
                          </div>
                        </div>
                      )}

                      {hasTask ? (
                        <div className="space-y-2 py-1">
                          {tasks.map((task, idx) => {
                            const taskIndex = schedule!.tasks.findIndex(
                              (t) => t.title === task.title && t.start_time === task.start_time
                            );
                            return (
                              <div
                                key={idx}
                                className={`${getTaskColor(task)} text-white rounded-lg shadow-md border-l-4 hover:shadow-lg transition-shadow ${
                                  draggedTask?.index === taskIndex ? 'opacity-50' : ''
                                }`}
                              >
                                {/* Content area */}
                                <div className="p-3 pb-2">
                                  <div className="flex items-center justify-between">
                                    <div className="flex items-center space-x-2">
                                      {/* Drag Handle Icon */}
                                      {onUpdateTask && (
                                        <span
                                          onMouseDown={(e) => handleMouseDown(e, task, taskIndex)}
                                          className="text-lg cursor-move select-none"
                                          title="드래그하여 이동"
                                        >
                                          ⋮⋮
                                        </span>
                                      )}
                                      <span className="text-lg">{getTaskIcon(task)}</span>
                                      <span className="font-semibold">{task.title}</span>
                                    </div>
                                    <span className="text-sm opacity-90 font-medium">
                                      {formatTime(task.start_time)} - {formatTime(task.end_time)}
                                    </span>
                                  </div>
                                  {task.tags.length > 0 && (
                                    <div className="text-xs opacity-90 mt-2 flex gap-2 ml-8">
                                      {task.tags.map((tag, i) => (
                                        <span key={i} className="bg-white bg-opacity-20 px-2 py-1 rounded">
                                          {tag}
                                        </span>
                                      ))}
                                    </div>
                                  )}
                                  <div className="text-xs opacity-75 mt-1 ml-8">
                                    {task.estimated_duration_minutes}분
                                  </div>
                                </div>

                                {/* Control buttons - NOT draggable */}
                                {onShiftSchedule && (
                                  <div className="px-3 pb-3 pt-0 flex justify-end gap-2">
                                    <button
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        openShiftModal(taskIndex, 'backward');
                                      }}
                                      className="bg-white bg-opacity-20 hover:bg-opacity-30 px-3 py-1 rounded text-xs font-semibold transition-colors"
                                      title="일정 당기기"
                                    >
                                      ⏪ 당기기
                                    </button>
                                    <button
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        openShiftModal(taskIndex, 'forward');
                                      }}
                                      className="bg-white bg-opacity-20 hover:bg-opacity-30 px-3 py-1 rounded text-xs font-semibold transition-colors"
                                      title="일정 미루기"
                                    >
                                      ⏩ 미루기
                                    </button>
                                  </div>
                                )}
                              </div>
                            );
                          })}
                        </div>
                      ) : (
                        <div className="h-[80px]" />
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        ) : (
          <div className="bg-white rounded-lg shadow-md p-12 text-center text-gray-500">
            <p className="text-lg">이 날짜에 사용 가능한 스케줄이 없습니다</p>
          </div>
        )}
      </div>

      {/* Shift Modal */}
      {shiftModalOpen && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
            <h3 className="text-xl font-bold text-gray-800 mb-4">
              {shiftDirection === 'forward' ? '⏩ 일정 미루기' : '⏪ 일정 당기기'}
            </h3>

            {/* Include current task checkbox */}
            <div className="mb-4">
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={includeCurrentTask}
                  onChange={(e) => setIncludeCurrentTask(e.target.checked)}
                  className="w-4 h-4"
                />
                <span className="text-gray-700">현재 작업 포함</span>
              </label>
              <p className="text-sm text-gray-500 ml-6 mt-1">
                {includeCurrentTask
                  ? '현재 작업부터 이후 모든 작업 시간 조정'
                  : '현재 작업 이후 작업들만 시간 조정'}
              </p>
            </div>

            {/* Time selection */}
            <div className="mb-6">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                {shiftDirection === 'forward' ? '미룰 시간' : '당길 시간'}
              </label>
              <div className="grid grid-cols-3 gap-2">
                {[5, 10, 15, 30, 60, 120].map((min) => (
                  <button
                    key={min}
                    onClick={() => setShiftMinutes(min)}
                    className={`px-4 py-2 rounded text-sm font-medium transition-colors ${
                      shiftMinutes === min
                        ? 'bg-blue-500 text-white'
                        : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                    }`}
                  >
                    {min >= 60 ? `${min / 60}시간` : `${min}분`}
                  </button>
                ))}
              </div>
            </div>

            {/* Action buttons */}
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setShiftModalOpen(false)}
                className="px-4 py-2 text-gray-700 bg-gray-100 hover:bg-gray-200 rounded font-medium transition-colors"
              >
                취소
              </button>
              <button
                onClick={handleConfirmShift}
                className="px-4 py-2 text-white bg-blue-500 hover:bg-blue-600 rounded font-medium transition-colors"
              >
                확인
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
