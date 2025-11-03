import { useEffect, useState, useRef } from 'react';
import type { PomodoroSession } from '../types';
import { api } from '../services/api';

interface PomodoroTimerProps {
  session: PomodoroSession;
}

export function PomodoroTimer({ session }: PomodoroTimerProps) {
  const [remainingSeconds, setRemainingSeconds] = useState<number>(0);
  const hasNotified = useRef(false);

  useEffect(() => {
    // Reset notification flag when session changes
    hasNotified.current = false;

    // current_start가 없으면 일시정지 상태
    if (!session.current_start) {
      return;
    }

    const calculateRemaining = () => {
      const startTime = new Date(session.current_start!);
      const now = new Date();
      const elapsedMs = now.getTime() - startTime.getTime();
      const elapsedSeconds = Math.floor(elapsedMs / 1000);
      const totalSeconds = session.pomodoro_duration * 60;
      const remainingSeconds = totalSeconds - elapsedSeconds;

      if (remainingSeconds < 0) {
        return 0;
      }

      return remainingSeconds;
    };

    // 초기 계산
    setRemainingSeconds(calculateRemaining());

    // 1초마다 업데이트
    const interval = setInterval(() => {
      const remaining = calculateRemaining();
      setRemainingSeconds(remaining);

      // 타이머가 0이 되면 알림
      if (remaining === 0 && !hasNotified.current) {
        hasNotified.current = true;
        api.sendNotification(
          '🍅 Pomodoro 완료!',
          `${session.pomodoro_duration}분 세션이 완료되었습니다. 잠시 휴식을 취하세요.`
        ).catch(console.error);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [session.current_start, session.pomodoro_duration]);

  const minutes = Math.floor(remainingSeconds / 60);
  const seconds = remainingSeconds % 60;

  // Pomodoro 진행률
  const totalSeconds = session.pomodoro_duration * 60;
  const progress = ((totalSeconds - remainingSeconds) / totalSeconds) * 100;

  return (
    <div className="flex items-center space-x-3">
      {/* 타이머 표시 */}
      <div className="flex items-center space-x-2">
        <div className="relative w-12 h-12">
          {/* 원형 진행바 */}
          <svg className="transform -rotate-90 w-12 h-12">
            <circle
              cx="24"
              cy="24"
              r="20"
              stroke="#e5e7eb"
              strokeWidth="3"
              fill="none"
            />
            <circle
              cx="24"
              cy="24"
              r="20"
              stroke="#22c55e"
              strokeWidth="3"
              fill="none"
              strokeDasharray={`${2 * Math.PI * 20}`}
              strokeDashoffset={`${2 * Math.PI * 20 * (1 - progress / 100)}`}
              className="transition-all duration-1000"
            />
          </svg>
          <div className="absolute inset-0 flex items-center justify-center">
            <span className="text-xs font-bold text-gray-700">
              {session.completed_pomodoros + 1}
            </span>
          </div>
        </div>

        <div className="flex flex-col">
          <span className="text-lg font-mono font-bold text-gray-800">
            {String(minutes).padStart(2, '0')}:{String(seconds).padStart(2, '0')}
          </span>
          <span className="text-xs text-gray-500">
            {session.completed_pomodoros}/{session.total_pomodoros} 🍅
          </span>
        </div>
      </div>
    </div>
  );
}
