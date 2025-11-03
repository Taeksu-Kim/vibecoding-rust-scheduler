import { useState } from 'react';
import type { Schedule, Task, TaskInput } from '../types';
import { PomodoroTimer } from './PomodoroTimer';
import { ClaudeModal } from './ClaudeModal';
import { api } from '../services/api';
import ReactMarkdown from 'react-markdown';

interface ScheduleViewProps {
  schedule: Schedule | null;
  selectedDate: string;
  onAddTask: (taskInput: TaskInput) => void;
  onUpdateTask: (index: number, taskInput: TaskInput) => void;
  onDeleteTask: (index: number) => void;
  onStartTask: (index: number) => void;
  onPauseTask: (index: number) => void;
  onResumeTask: (index: number) => void;
  onCompleteTask: (index: number, focusScore: number) => void;
}

export function ScheduleView({
  schedule,
  selectedDate,
  onAddTask,
  onDeleteTask,
  onStartTask,
  onPauseTask,
  onResumeTask,
  onCompleteTask,
}: ScheduleViewProps) {
  const [showAddModal, setShowAddModal] = useState(false);
  const [showClaudeModal, setShowClaudeModal] = useState(false);
  const [claudeResponse, setClaudeResponse] = useState<string>('');
  const [isEvaluating, setIsEvaluating] = useState(false);
  const [aiProvider, setAiProvider] = useState<string>('claude'); // 'claude' or 'copilot'
  const [aiProviderStatus, setAiProviderStatus] = useState<{ [key: string]: boolean }>({});
  const [showInstallGuide, setShowInstallGuide] = useState(false);
  const [installGuideText, setInstallGuideText] = useState<string>('');

  const totalTasks = schedule?.tasks.length || 0;
  const completedTasks = schedule?.tasks.filter(t => t.status === 'Completed').length || 0;
  const completionRate = schedule?.completion_rate ?? (totalTasks > 0 ? (completedTasks / totalTasks) * 100 : 0);

  // AI 프로바이더 변경 핸들러
  const handleProviderChange = async (newProvider: string) => {
    setAiProvider(newProvider);

    // 선택한 프로바이더 사용 가능 여부 확인
    try {
      const result = await api.checkAIProvider(newProvider);
      console.log(`${newProvider} status:`, result);
      setAiProviderStatus(prev => ({ ...prev, [newProvider]: true }));
    } catch (error) {
      console.error(`${newProvider} not available:`, error);
      setAiProviderStatus(prev => ({ ...prev, [newProvider]: false }));

      // 설치 안내 표시
      try {
        const guide = await api.getAIInstallationGuide(newProvider);
        setInstallGuideText(guide);
        setShowInstallGuide(true);
      } catch (e) {
        console.error('Failed to get installation guide:', e);
      }
    }
  };

  const handleEvaluateSchedule = async () => {
    if (!schedule || schedule.tasks.length === 0) {
      alert('평가할 스케줄이 없습니다.');
      return;
    }

    setIsEvaluating(true);
    setShowClaudeModal(true);
    const providerName = aiProvider === 'copilot' ? 'Copilot' : 'Claude';
    setClaudeResponse(`${providerName}이(가) 스케줄을 분석하고 있습니다...`);

    try {
      // Build schedule description
      let prompt = `오늘(${selectedDate}) 내 스케줄:\n`;
      schedule.tasks.forEach((task, i) => {
        const start = new Date(task.start_time).toLocaleTimeString('ko-KR', { hour: '2-digit', minute: '2-digit', hour12: false });
        const end = new Date(task.end_time).toLocaleTimeString('ko-KR', { hour: '2-digit', minute: '2-digit', hour12: false });
        prompt += `${i + 1}. ${task.title} (${start}-${end}, ${task.estimated_duration_minutes}분)\n`;
      });

      prompt += '\n이 스케줄이 현실적인지 평가하고, 개선점이 있다면 구체적으로 알려주세요. 특히 다음 사항들을 고려해주세요:\n';
      prompt += '1. 작업 간 전환 시간이 충분한가?\n';
      prompt += '2. 각 작업의 예상 시간이 현실적인가?\n';
      prompt += '3. 휴식 시간이 적절한가?\n';
      prompt += '4. 집중력 리듬을 고려했을 때 작업 순서가 적절한가?';

      const response = await api.askAI(prompt, aiProvider);
      setClaudeResponse(response);
    } catch (err) {
      setClaudeResponse(`오류가 발생했습니다: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsEvaluating(false);
    }
  };

  return (
    <div className="flex-1 bg-gray-50 overflow-auto">
      <div className="max-w-6xl mx-auto p-8">
        {/* Header */}
        <div className="flex justify-between items-center mb-6">
          <div>
            <h2 className="text-3xl font-bold text-gray-800">
              {new Date(selectedDate).toLocaleDateString('en-US', {
                weekday: 'long',
                year: 'numeric',
                month: 'long',
                day: 'numeric',
              })}
            </h2>
            {schedule && (
              <p className="text-gray-600 mt-1">
                {totalTasks} tasks • {completionRate.toFixed(0)}% complete
              </p>
            )}
          </div>
          <div className="flex items-center space-x-3">
            {/* AI Provider Selector */}
            <select
              value={aiProvider}
              onChange={(e) => handleProviderChange(e.target.value)}
              className="border border-gray-300 rounded-lg px-4 py-2 bg-white text-gray-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="claude">
                🤖 Claude {aiProviderStatus.claude === false && '(미설치)'}
              </option>
              <option value="copilot">
                🤖 Copilot {aiProviderStatus.copilot === false && '(미설치)'}
              </option>
            </select>

            <button
              onClick={handleEvaluateSchedule}
              disabled={!schedule || schedule.tasks.length === 0}
              className="bg-blue-500 hover:bg-blue-600 text-white font-medium px-6 py-3 rounded-lg transition-colors flex items-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <span>🤖</span>
              <span>AI 평가</span>
            </button>
            <button
              onClick={() => setShowAddModal(true)}
              className="bg-primary-500 hover:bg-primary-600 text-white font-medium px-6 py-3 rounded-lg transition-colors flex items-center space-x-2"
            >
              <span className="text-xl">+</span>
              <span>작업 추가</span>
            </button>
          </div>
        </div>

        {/* Task List */}
        <div className="bg-white rounded-lg shadow-md">
          {schedule && schedule.tasks.length > 0 ? (
            <div className="divide-y divide-gray-200">
              {schedule.tasks.map((task, index) => (
                <TaskItem
                  key={task.id}
                  task={task}
                  index={index}
                  onDelete={() => onDeleteTask(index)}
                  onStart={() => onStartTask(index)}
                  onPause={() => onPauseTask(index)}
                  onResume={() => onResumeTask(index)}
                  onComplete={(score) => onCompleteTask(index, score)}
                />
              ))}
            </div>
          ) : (
            <div className="p-12 text-center text-gray-500">
              <p className="text-lg mb-2">이 날짜에 예정된 작업이 없습니다</p>
              <p className="text-sm">"작업 추가" 버튼을 클릭하여 시작하세요</p>
            </div>
          )}
        </div>
      </div>

      {/* Add Task Modal */}
      {showAddModal && (
        <TaskModal
          selectedDate={selectedDate}
          aiProvider={aiProvider}
          onClose={() => setShowAddModal(false)}
          onSave={(taskInput) => {
            onAddTask(taskInput);
            setShowAddModal(false);
          }}
        />
      )}

      {/* Claude Evaluation Modal */}
      {showClaudeModal && (
        <ClaudeModal
          response={claudeResponse}
          isLoading={isEvaluating}
          onClose={() => setShowClaudeModal(false)}
        />
      )}

      {/* AI Installation Guide Modal */}
      {showInstallGuide && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full p-6">
            <div className="flex justify-between items-start mb-4">
              <h3 className="text-xl font-bold text-gray-800">AI CLI 설치 안내</h3>
              <button
                onClick={() => setShowInstallGuide(false)}
                className="text-gray-400 hover:text-gray-600 text-2xl leading-none"
              >
                ×
              </button>
            </div>
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
              <p className="text-sm text-yellow-800">
                ⚠️ 선택한 AI 프로바이더가 설치되지 않았거나 사용할 수 없습니다.
              </p>
            </div>
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
              <pre className="whitespace-pre-wrap text-sm text-gray-700 font-mono">
                {installGuideText}
              </pre>
            </div>
            <div className="mt-6 flex justify-end">
              <button
                onClick={() => setShowInstallGuide(false)}
                className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
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

interface TaskItemProps {
  task: Task;
  index: number;
  onDelete: () => void;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onComplete: (score: number) => void;
}

function TaskItem({ task, onDelete, onStart, onPause, onResume, onComplete }: TaskItemProps) {
  const [showCompleteModal, setShowCompleteModal] = useState(false);

  const formatTime = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
  };

  return (
    <div className="p-4 hover:bg-gray-50 transition-colors">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4 flex-1">
          {/* Completion Status */}
          <div className="flex items-center">
            {task.status === 'Completed' ? (
              <div className="w-6 h-6 bg-green-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm">✓</span>
              </div>
            ) : task.status === 'InProgress' ? (
              <div className="w-6 h-6 bg-blue-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm">▶</span>
              </div>
            ) : task.status === 'Paused' ? (
              <div className="w-6 h-6 bg-yellow-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm">⏸</span>
              </div>
            ) : (
              <div className="w-6 h-6 border-2 border-gray-300 rounded-full" />
            )}
          </div>

          {/* Task Info */}
          <div className="flex-1">
            <div className="flex items-center space-x-3 mb-1">
              <span className="font-medium text-gray-800">{task.title}</span>
              {task.tags.length > 0 && (
                <div className="flex gap-1">
                  {task.tags.map((tag, idx) => (
                    <span key={idx} className="text-xs px-2 py-1 rounded bg-gray-100 text-gray-700">
                      {tag}
                    </span>
                  ))}
                </div>
              )}
            </div>
            <div className="text-sm text-gray-600">
              {formatTime(task.start_time)} - {formatTime(task.end_time)}
              <span className="ml-3">({task.estimated_duration_minutes}m)</span>
            </div>
            {task.notes && (
              <p className="text-sm text-gray-500 mt-1">{task.notes}</p>
            )}
            {/* Pomodoro Timer */}
            {task.status === 'InProgress' && task.pomodoro && (
              <div className="mt-2">
                <PomodoroTimer session={task.pomodoro} />
              </div>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center space-x-2">
          {task.status === 'Pending' && (
            <button
              onClick={onStart}
              className="px-4 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded transition-colors text-sm font-medium"
            >
              Start
            </button>
          )}
          {task.status === 'InProgress' && (
            <>
              <button
                onClick={onPause}
                className="px-4 py-2 bg-yellow-500 hover:bg-yellow-600 text-white rounded transition-colors text-sm font-medium"
              >
                Pause
              </button>
              <button
                onClick={() => setShowCompleteModal(true)}
                className="px-4 py-2 bg-green-500 hover:bg-green-600 text-white rounded transition-colors text-sm font-medium"
              >
                Complete
              </button>
            </>
          )}
          {task.status === 'Paused' && (
            <button
              onClick={onResume}
              className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded transition-colors text-sm font-medium"
            >
              Resume
            </button>
          )}
          {task.status !== 'Completed' && (
            <button
              onClick={onDelete}
              className="px-3 py-1 text-red-600 hover:bg-red-50 rounded transition-colors text-sm"
            >
              Delete
            </button>
          )}
        </div>
      </div>

      {/* Complete Modal */}
      {showCompleteModal && (
        <CompleteModal
          onClose={() => setShowCompleteModal(false)}
          onComplete={(score) => {
            onComplete(score);
            setShowCompleteModal(false);
          }}
        />
      )}
    </div>
  );
}

interface TaskModalProps {
  onClose: () => void;
  onSave: (taskInput: TaskInput) => void;
  selectedDate: string;
  aiProvider: string;
}

function TaskModal({ onClose, onSave, selectedDate, aiProvider }: TaskModalProps) {
  const [title, setTitle] = useState('');
  const [startTime, setStartTime] = useState('09:00');
  const [endTime, setEndTime] = useState('10:00');
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');
  const [notes, setNotes] = useState('');
  const [pomodoroDuration, setPomodoroDuration] = useState<number>(25);
  const [showPomodoroGuide, setShowPomodoroGuide] = useState(false);
  const [showClaudeAdvice, setShowClaudeAdvice] = useState(false);
  const [claudeAdvice, setClaudeAdvice] = useState<string>('');
  const [isGettingAdvice, setIsGettingAdvice] = useState(false);
  const [isAutoCompleting, setIsAutoCompleting] = useState(false);
  const [showAutoCompleteInput, setShowAutoCompleteInput] = useState(false);
  const [autoCompleteInput, setAutoCompleteInput] = useState('');
  const [autoCompleteReasoning, setAutoCompleteReasoning] = useState<string>('');
  const [showAutoCompleteReasoning, setShowAutoCompleteReasoning] = useState(false);

  const handleAutoComplete = async () => {
    if (!autoCompleteInput.trim()) {
      alert('작업 내용을 입력해주세요.');
      return;
    }

    setIsAutoCompleting(true);

    try {
      const suggestion = await api.suggestTaskCompletion(selectedDate, autoCompleteInput, aiProvider);

      // 모든 필드 자동 채우기 (제목 포함)
      setTitle(suggestion.suggested_title);
      setStartTime(suggestion.suggested_start_time);
      setEndTime(suggestion.suggested_end_time);
      setTags(suggestion.tags);
      setNotes(suggestion.notes || '');
      setPomodoroDuration(suggestion.pomodoro_duration);

      // AI 고려 사항 표시 (별도 영역)
      setAutoCompleteReasoning(suggestion.reasoning);
      setShowAutoCompleteReasoning(true);

      // 입력 필드 숨기기
      setShowAutoCompleteInput(false);
      setAutoCompleteInput('');
    } catch (err) {
      alert(`자동 완성 실패: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsAutoCompleting(false);
    }
  };

  const calculateDuration = () => {
    const [startH, startM] = startTime.split(':').map(Number);
    const [endH, endM] = endTime.split(':').map(Number);
    const startMinutes = startH * 60 + startM;
    const endMinutes = endH * 60 + endM;
    return endMinutes - startMinutes;
  };

  const handleGetAdvice = async () => {
    if (!title.trim()) {
      alert('작업 제목을 먼저 입력해주세요.');
      return;
    }

    const duration = calculateDuration();
    if (duration <= 0) {
      alert('종료 시간이 시작 시간보다 빠릅니다.');
      return;
    }

    setIsGettingAdvice(true);
    setShowClaudeAdvice(true);
    const providerName = aiProvider === 'copilot' ? 'Copilot' : 'Claude';
    setClaudeAdvice(`${providerName}이(가) 작업을 분석하고 있습니다...`);

    try {
      const advice = await api.getTaskAdvice(title, duration, aiProvider);
      setClaudeAdvice(advice);
    } catch (err) {
      setClaudeAdvice(`오류가 발생했습니다: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsGettingAdvice(false);
    }
  };

  const handleSave = () => {
    if (!title.trim()) return;

    const taskInput: TaskInput = {
      title: title.trim(),
      start_time: startTime,
      end_time: endTime,
      tags,
      notes: notes.trim() || undefined,
      pomodoro_duration: pomodoroDuration,
    };

    onSave(taskInput);
  };

  const addTag = () => {
    if (tagInput.trim() && !tags.includes(tagInput.trim())) {
      setTags([...tags, tagInput.trim()]);
      setTagInput('');
    }
  };

  const removeTag = (tag: string) => {
    setTags(tags.filter(t => t !== tag));
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full max-h-[90vh] flex flex-col">
        <div className="p-6 border-b border-gray-200 flex-shrink-0">
          <h3 className="text-xl font-bold text-gray-800">새 작업 추가</h3>
        </div>

        <div className="p-6 space-y-4 overflow-y-auto flex-1">
          {/* AI Auto Complete Section */}
          <div className="bg-gradient-to-br from-purple-50 to-blue-50 border-2 border-purple-200 rounded-lg p-4">
            {!showAutoCompleteInput ? (
              <div className="flex flex-col items-center space-y-2">
                <p className="text-sm text-gray-700 text-center">
                  AI가 자동으로 작업을 분석하고 모든 필드를 채워드립니다
                </p>
                <button
                  type="button"
                  onClick={() => setShowAutoCompleteInput(true)}
                  className="px-6 py-3 bg-gradient-to-r from-purple-500 to-blue-500 hover:from-purple-600 hover:to-blue-600 text-white font-semibold rounded-lg transition-all shadow-md hover:shadow-lg flex items-center space-x-2"
                >
                  <span>✨</span>
                  <span>AI 자동 완성</span>
                </button>
              </div>
            ) : (
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <label className="text-sm font-semibold text-gray-700">
                    자연어로 작업을 설명해주세요
                  </label>
                  <button
                    type="button"
                    onClick={() => {
                      setShowAutoCompleteInput(false);
                      setAutoCompleteInput('');
                    }}
                    className="text-gray-400 hover:text-gray-600 text-xl"
                  >
                    ×
                  </button>
                </div>
                <textarea
                  value={autoCompleteInput}
                  onChange={(e) => setAutoCompleteInput(e.target.value)}
                  placeholder="예: 조깅 30분 아침에&#10;예: 코드 리뷰 1시간 오전 중에&#10;예: API 개발 2시간 오후에"
                  className="w-full px-3 py-2 border-2 border-purple-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent resize-none"
                  rows={3}
                  autoFocus
                />
                <button
                  type="button"
                  onClick={handleAutoComplete}
                  disabled={!autoCompleteInput.trim() || isAutoCompleting}
                  className="w-full px-4 py-2 bg-gradient-to-r from-purple-500 to-blue-500 hover:from-purple-600 hover:to-blue-600 text-white font-semibold rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2"
                >
                  {isAutoCompleting ? (
                    <>
                      <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                      <span>AI 분석 중...</span>
                    </>
                  ) : (
                    <>
                      <span>✨</span>
                      <span>자동 완성 실행</span>
                    </>
                  )}
                </button>
              </div>
            )}
          </div>

          {/* AI 고려 사항 (자동 완성 후 표시) */}
          {showAutoCompleteReasoning && (
            <div className="bg-purple-50 border border-purple-200 rounded-lg p-4">
              <div className="flex justify-between items-start mb-2">
                <div className="flex items-center space-x-2">
                  <span className="text-lg">✨</span>
                  <span className="font-semibold text-purple-800">AI가 고려한 사항</span>
                </div>
                <button
                  type="button"
                  onClick={() => setShowAutoCompleteReasoning(false)}
                  className="text-purple-400 hover:text-purple-600 text-xl leading-none"
                >
                  ×
                </button>
              </div>
              <div className="prose prose-sm max-w-none prose-p:text-purple-900 prose-p:text-sm prose-p:leading-relaxed">
                <ReactMarkdown>{autoCompleteReasoning}</ReactMarkdown>
              </div>
            </div>
          )}

          {/* Title */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Title
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g., 조깅, 코드 리뷰, API 개발..."
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            />
          </div>

          {/* Time Range */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Start Time
              </label>
              <input
                type="time"
                value={startTime}
                onChange={(e) => setStartTime(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                End Time
              </label>
              <input
                type="time"
                value={endTime}
                onChange={(e) => setEndTime(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              />
            </div>
          </div>

          {/* Tags */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Tags
            </label>
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
                placeholder="태그 추가..."
                className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              />
              <button
                onClick={addTag}
                className="px-4 py-2 bg-gray-200 hover:bg-gray-300 rounded-lg transition-colors"
              >
                추가
              </button>
            </div>
            {tags.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {tags.map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center gap-1 px-2 py-1 bg-primary-100 text-primary-700 rounded text-sm"
                  >
                    {tag}
                    <button
                      onClick={() => removeTag(tag)}
                      className="hover:text-primary-900"
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            )}
          </div>

          {/* Notes */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Notes (optional)
            </label>
            <textarea
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="메모를 입력하세요..."
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            />
          </div>

          {/* Pomodoro Duration */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm font-medium text-gray-700">
                집중 세션 시간 (Pomodoro)
              </label>
              <button
                type="button"
                onClick={() => setShowPomodoroGuide(!showPomodoroGuide)}
                className="text-xs text-blue-600 hover:text-blue-700"
              >
                {showPomodoroGuide ? '가이드 숨기기' : '가이드 보기'}
              </button>
            </div>

            {showPomodoroGuide && (
              <div className="mb-3 p-3 bg-blue-50 border border-blue-200 rounded-lg text-xs text-gray-700 space-y-2">
                <p className="font-semibold text-blue-800">Pomodoro 기법이란?</p>
                <p>작업을 일정한 시간 단위로 나눠서 집중력을 유지하는 시간 관리 기법입니다.</p>

                <div className="mt-2">
                  <p className="font-semibold text-blue-800">장점:</p>
                  <ul className="list-disc list-inside space-y-1 ml-2">
                    <li>집중력 향상 및 번아웃 방지</li>
                    <li>진행도를 시각적으로 확인 가능</li>
                    <li>작업 완료 시 성취감 증대</li>
                  </ul>
                </div>

                <div className="mt-2">
                  <p className="font-semibold text-blue-800">추천 설정:</p>
                  <ul className="list-disc list-inside space-y-1 ml-2">
                    <li><strong>15-20분:</strong> 단순 반복 작업 (이메일, 문서 정리)</li>
                    <li><strong>25분:</strong> 일반적인 작업 (표준 Pomodoro)</li>
                    <li><strong>45-60분:</strong> 깊은 집중이 필요한 작업 (코딩, 글쓰기)</li>
                    <li><strong>90분:</strong> 창의적이고 복잡한 작업 (설계, 연구)</li>
                  </ul>
                </div>
              </div>
            )}

            <div className="flex items-center space-x-3">
              <input
                type="range"
                min="5"
                max="90"
                step="5"
                value={pomodoroDuration}
                onChange={(e) => setPomodoroDuration(Number(e.target.value))}
                className="flex-1"
              />
              <div className="flex items-center space-x-2">
                <input
                  type="number"
                  min="5"
                  max="90"
                  step="5"
                  value={pomodoroDuration}
                  onChange={(e) => {
                    const val = Number(e.target.value);
                    if (val >= 5 && val <= 90) {
                      setPomodoroDuration(val);
                    }
                  }}
                  className="w-16 px-2 py-1 border border-gray-300 rounded text-center"
                />
                <span className="text-sm text-gray-600">분</span>
              </div>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              이 작업을 {pomodoroDuration}분 단위로 나눠서 집중합니다
            </p>
          </div>
        </div>

        {/* Claude Advice - Above buttons */}
        {showClaudeAdvice && (
          <div className="px-6 pb-4 flex-shrink-0">
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
              <div className="flex justify-between items-start mb-2">
                <div className="flex items-center space-x-2">
                  <span className="text-lg">🤖</span>
                  <span className="font-semibold text-blue-800">Claude의 조언</span>
                </div>
                {!isGettingAdvice && (
                  <button
                    onClick={() => setShowClaudeAdvice(false)}
                    className="text-blue-400 hover:text-blue-600 text-xl leading-none"
                  >
                    ×
                  </button>
                )}
              </div>
              {isGettingAdvice ? (
                <div className="flex items-center space-x-2 text-sm text-blue-600">
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
                  <span>분석 중...</span>
                </div>
              ) : (
                <div className="prose prose-sm max-w-none prose-headings:text-blue-800 prose-p:text-blue-900 prose-p:text-sm prose-p:leading-relaxed prose-ul:list-disc prose-ul:ml-4 prose-li:text-blue-900 prose-li:text-sm prose-strong:text-blue-950 prose-strong:font-semibold prose-code:bg-blue-100 prose-code:px-1 prose-code:rounded prose-code:text-xs">
                  <ReactMarkdown>{claudeAdvice}</ReactMarkdown>
                </div>
              )}
            </div>
          </div>
        )}

        <div className="p-6 border-t border-gray-200 flex justify-between items-center flex-shrink-0">
          <button
            type="button"
            onClick={handleGetAdvice}
            disabled={!title.trim()}
            className="px-4 py-2 bg-blue-100 text-blue-700 hover:bg-blue-200 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center space-x-2"
          >
            <span>🤖</span>
            <span>AI 조언</span>
          </button>

          <div className="flex space-x-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
            >
              취소
            </button>
            <button
              onClick={handleSave}
              disabled={!title.trim()}
              className="px-4 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              작업 저장
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

interface CompleteModalProps {
  onClose: () => void;
  onComplete: (score: number) => void;
}

function CompleteModal({ onClose, onComplete }: CompleteModalProps) {
  const [focusScore, setFocusScore] = useState(7);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-sm w-full mx-4">
        <div className="p-6 border-b border-gray-200">
          <h3 className="text-xl font-bold text-gray-800">Complete Task</h3>
        </div>

        <div className="p-6">
          <label className="block text-sm font-medium text-gray-700 mb-3">
            How well did you focus? (1-10)
          </label>
          <div className="flex items-center space-x-4">
            <input
              type="range"
              min="1"
              max="10"
              value={focusScore}
              onChange={(e) => setFocusScore(Number(e.target.value))}
              className="flex-1"
            />
            <span className="text-2xl font-bold text-primary-600 w-12 text-center">
              {focusScore}
            </span>
          </div>
          <p className="text-sm text-gray-500 mt-2">
            Higher scores increase your efficiency rating
          </p>
        </div>

        <div className="p-6 border-t border-gray-200 flex justify-end space-x-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
          >
            취소
          </button>
          <button
            onClick={() => onComplete(focusScore)}
            className="px-4 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-lg transition-colors"
          >
            완료 표시
          </button>
        </div>
      </div>
    </div>
  );
}
