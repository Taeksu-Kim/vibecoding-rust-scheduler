import { useState, useEffect } from 'react';
import type { Schedule } from '../types';

interface DashboardViewProps {
  schedule: Schedule | null;
  selectedDate: string;
}

export function DashboardView({ schedule, selectedDate }: DashboardViewProps) {
  const [currentTime, setCurrentTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);
    return () => clearInterval(timer);
  }, []);
  if (!schedule) {
    return (
      <div className="flex-1 bg-gray-50 overflow-auto">
        <div className="max-w-6xl mx-auto p-8">
          <h2 className="text-3xl font-bold text-gray-800 mb-6">Dashboard</h2>
          <div className="bg-white rounded-lg shadow-md p-12 text-center text-gray-500">
            <p className="text-lg">No data available for this date</p>
          </div>
        </div>
      </div>
    );
  }

  const totalTasks = schedule.tasks.length;
  const completedTasks = schedule.tasks.filter((t) => t.status === 'Completed').length;
  const completionRate = schedule.completion_rate ?? (totalTasks > 0 ? (completedTasks / totalTasks) * 100 : 0);

  const efficiencyScore = schedule.efficiency_score ?? completionRate;
  const totalEarned = schedule.total_earned ?? 0;
  const totalWasted = schedule.total_wasted ?? 0;
  const totalBonus = schedule.total_bonus ?? 0;
  const totalPenalty = schedule.total_penalty ?? 0;

  const tagStats = schedule.tasks.reduce((acc, task) => {
    task.tags.forEach(tag => {
      acc[tag] = (acc[tag] || 0) + 1;
    });
    return acc;
  }, {} as Record<string, number>);

  const statusStats = schedule.tasks.reduce((acc, task) => {
    acc[task.status] = (acc[task.status] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  // Average duration instead of focus score
  const avgDuration =
    schedule.tasks
      .filter((t) => t.actual_duration_minutes !== undefined)
      .reduce((sum, t) => sum + (t.actual_duration_minutes || 0), 0) /
      schedule.tasks.filter((t) => t.actual_duration_minutes !== undefined).length || 0;

  const getGrade = (score: number): string => {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  };

  const grade = getGrade(efficiencyScore);
  const gradeColor =
    grade === 'A' || grade === 'B'
      ? 'text-green-600'
      : grade === 'C'
      ? 'text-yellow-600'
      : 'text-red-600';

  return (
    <div className="flex-1 bg-gray-50 overflow-auto">
      <div className="max-w-6xl mx-auto p-8">
        <div className="mb-8">
          <h1 className="text-4xl font-bold text-gray-800 mb-2">📊 대시보드</h1>
          <p className="text-gray-600">
            {new Date(selectedDate).toLocaleDateString('ko-KR', {
              weekday: 'long',
              year: 'numeric',
              month: 'long',
              day: 'numeric',
            })}
          </p>
          <p className="text-sm text-gray-500 mt-1">
            현재 시간: {currentTime.toLocaleTimeString('ko-KR')}
          </p>
        </div>

        {/* Main Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-sm font-medium text-gray-600 mb-2">효율 점수</h3>
            <div className="flex items-baseline space-x-2">
              <span className="text-4xl font-bold text-primary-600">
                {efficiencyScore.toFixed(1)}%
              </span>
              <span className={`text-3xl font-bold ${gradeColor}`}>{grade}</span>
            </div>
            <div className="mt-3 bg-gray-200 rounded-full h-2">
              <div
                className="bg-primary-500 rounded-full h-2 transition-all"
                style={{ width: `${efficiencyScore}%` }}
              />
            </div>
          </div>

          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-sm font-medium text-gray-600 mb-2">작업 완료</h3>
            <div className="flex items-baseline space-x-2">
              <span className="text-4xl font-bold text-gray-800">
                {completedTasks}/{totalTasks}
              </span>
            </div>
            <p className="text-sm text-gray-600 mt-2">
              {completionRate.toFixed(0)}% 완료
            </p>
            <div className="mt-3 bg-gray-200 rounded-full h-2">
              <div
                className="bg-green-500 rounded-full h-2 transition-all"
                style={{ width: `${completionRate}%` }}
              />
            </div>
          </div>

          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-sm font-medium text-gray-600 mb-2">평균 소요 시간</h3>
            <div className="flex items-baseline space-x-2">
              <span className="text-4xl font-bold text-gray-800">
                {avgDuration > 0 ? avgDuration.toFixed(0) : 'N/A'}
              </span>
              {avgDuration > 0 && (
                <span className="text-xl text-gray-600">분</span>
              )}
            </div>
            <p className="text-sm text-gray-600 mt-2">
              완료된 작업당
            </p>
          </div>
        </div>

        {/* Time Stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <MetricCard
            label="획득한 시간"
            value={`${totalEarned}분`}
            color="green"
          />
          <MetricCard
            label="낭비한 시간"
            value={`${totalWasted}분`}
            color="red"
          />
          <MetricCard
            label="보너스 시간"
            value={`+${totalBonus}분`}
            color="blue"
          />
          <MetricCard
            label="페널티 시간"
            value={`-${totalPenalty}분`}
            color="orange"
          />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Tags Breakdown */}
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-bold text-gray-800 mb-4">태그별 작업</h3>
            <div className="space-y-3">
              {Object.keys(tagStats).length > 0 ? (
                Object.entries(tagStats)
                  .sort(([, a], [, b]) => b - a)
                  .map(([tag, count]) => (
                    <div key={tag}>
                      <div className="flex justify-between items-center mb-1">
                        <span className="text-sm font-medium text-gray-700">{tag}</span>
                        <span className="text-sm text-gray-600">{count}개</span>
                      </div>
                      <div className="bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-primary-500 rounded-full h-2"
                          style={{ width: `${(count / totalTasks) * 100}%` }}
                        />
                      </div>
                    </div>
                  ))
              ) : (
                <p className="text-gray-500 text-sm">할당된 태그 없음</p>
              )}
            </div>
          </div>

          {/* Status Breakdown */}
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-bold text-gray-800 mb-4">상태별 작업</h3>
            <div className="space-y-3">
              {Object.entries(statusStats)
                .sort(([, a], [, b]) => b - a)
                .map(([status, count]) => {
                  const colors: Record<string, string> = {
                    Completed: 'bg-green-500',
                    InProgress: 'bg-blue-500',
                    Pending: 'bg-yellow-500',
                    Paused: 'bg-orange-500',
                    Skipped: 'bg-gray-500',
                  };
                  return (
                    <div key={status}>
                      <div className="flex justify-between items-center mb-1">
                        <span className="text-sm font-medium text-gray-700">{status}</span>
                        <span className="text-sm text-gray-600">{count}</span>
                      </div>
                      <div className="bg-gray-200 rounded-full h-2">
                        <div
                          className={`${colors[status] || 'bg-gray-500'} rounded-full h-2`}
                          style={{ width: `${(count / totalTasks) * 100}%` }}
                        />
                      </div>
                    </div>
                  );
                })}
            </div>
          </div>
        </div>

        {/* Schedule Change History */}
        {schedule.changes && schedule.changes.length > 0 && (
          <div className="mt-6 bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-bold text-gray-800 mb-4">
              📝 스케줄 변경 이력 ({schedule.changes.length})
            </h3>
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {schedule.changes.slice().reverse().map((change, index) => {
                const changeTime = new Date(change.timestamp);
                const timeStr = changeTime.toLocaleTimeString('ko-KR', {
                  hour: '2-digit',
                  minute: '2-digit',
                  second: '2-digit'
                });

                const getChangeIcon = (type: string) => {
                  switch (type) {
                    case 'TaskMoved': return '🔄';
                    case 'ScheduleShifted': return '⏩';
                    case 'TaskUpdated': return '✏️';
                    case 'TaskCreated': return '➕';
                    case 'TaskDeleted': return '🗑️';
                    default: return '📝';
                  }
                };

                const getChangeColor = (type: string) => {
                  switch (type) {
                    case 'TaskMoved': return 'bg-blue-50 border-blue-200';
                    case 'ScheduleShifted': return 'bg-purple-50 border-purple-200';
                    case 'TaskUpdated': return 'bg-yellow-50 border-yellow-200';
                    case 'TaskCreated': return 'bg-green-50 border-green-200';
                    case 'TaskDeleted': return 'bg-red-50 border-red-200';
                    default: return 'bg-gray-50 border-gray-200';
                  }
                };

                return (
                  <div
                    key={index}
                    className={`${getChangeColor(change.change_type)} border rounded-lg p-3 hover:shadow-md transition-shadow`}
                  >
                    <div className="flex items-start space-x-3">
                      <span className="text-2xl">{getChangeIcon(change.change_type)}</span>
                      <div className="flex-1">
                        <p className="text-sm font-medium text-gray-800">{change.description}</p>
                        <div className="flex items-center space-x-2 mt-1">
                          <span className="text-xs text-gray-500">{timeStr}</span>
                          {change.affected_tasks_count && (
                            <span className="text-xs bg-white bg-opacity-50 px-2 py-0.5 rounded">
                              영향: {change.affected_tasks_count}개
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Psychology Principles Applied */}
        <div className="mt-6 bg-gradient-to-r from-primary-50 to-green-50 rounded-lg shadow-md p-6">
          <h3 className="text-lg font-bold text-gray-800 mb-4">
            🧠 적용된 15가지 심리학 원리 및 시간 관리 방법론
          </h3>
          <p className="text-sm text-gray-600 mb-4">
            이 스케줄러는 과학적으로 검증된 심리학 이론과 시간 관리 기법을 바탕으로 설계되었습니다.
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 목표 설정 이론 (Goal Setting Theory)</div>
              <div className="text-gray-600">구체적인 시간과 작업을 미리 정의하여 명확한 목표를 설정합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 실행 의도 (Implementation Intentions)</div>
              <div className="text-gray-600">"언제, 어디서, 무엇을" 미리 계획하여 실행 가능성을 높입니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 타임 블로킹 (Time Blocking)</div>
              <div className="text-gray-600">하루를 작업 단위로 나누어 집중력을 극대화합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 포모도로 기법 (Pomodoro Technique)</div>
              <div className="text-gray-600">25분 집중 + 5분 휴식 사이클로 지속 가능한 생산성을 유지합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 몰입 상태 (Flow State)</div>
              <div className="text-gray-600">적절한 난이도의 작업 배치로 최적의 집중 상태를 유도합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 게이미피케이션 (Gamification)</div>
              <div className="text-gray-600">점수, 등급, 보너스 시스템으로 동기부여를 강화합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 일관성 원리 (Commitment & Consistency)</div>
              <div className="text-gray-600">계획을 시각화하여 약속을 지키려는 심리적 압박을 활용합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 손실 회피 (Loss Aversion)</div>
              <div className="text-gray-600">낭비된 시간을 명시적으로 표시하여 손실에 대한 민감도를 높입니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 진행 상황 추적 (Progress Tracking)</div>
              <div className="text-gray-600">실시간으로 완료율과 효율 점수를 확인하여 성취감을 제공합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 즉각적 피드백 (Immediate Feedback)</div>
              <div className="text-gray-600">작업 완료 시 즉시 통계가 업데이트되어 행동 강화 효과를 제공합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 습관 형성 (Habit Formation)</div>
              <div className="text-gray-600">매일 반복되는 스케줄 패턴으로 생산적인 습관을 형성합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 에너지 관리 (Energy Management)</div>
              <div className="text-gray-600">휴식 시간 배치로 장기적인 에너지 수준을 유지합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 희소성 원리 (Scarcity Principle)</div>
              <div className="text-gray-600">남은 시간을 명시하여 시간의 유한성을 인식하고 집중력을 높입니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 파킨슨의 법칙 (Parkinson's Law)</div>
              <div className="text-gray-600">명확한 시간 제약으로 작업 확장을 방지하고 효율성을 극대화합니다.</div>
            </div>
            <div className="bg-white bg-opacity-50 p-3 rounded">
              <div className="font-semibold text-gray-800 mb-1">✓ 자이가르닉 효과 (Zeigarnik Effect)</div>
              <div className="text-gray-600">미완료 작업을 강조하여 완료하려는 심리적 동기를 유발합니다.</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface MetricCardProps {
  label: string;
  value: string;
  color: 'green' | 'red' | 'blue' | 'orange';
}

function MetricCard({ label, value, color }: MetricCardProps) {
  const colorClasses = {
    green: 'bg-green-50 text-green-700 border-green-200',
    red: 'bg-red-50 text-red-700 border-red-200',
    blue: 'bg-blue-50 text-blue-700 border-blue-200',
    orange: 'bg-orange-50 text-orange-700 border-orange-200',
  };

  return (
    <div className={`${colorClasses[color]} border rounded-lg p-4`}>
      <p className="text-xs font-medium opacity-80">{label}</p>
      <p className="text-2xl font-bold mt-1">{value}</p>
    </div>
  );
}
