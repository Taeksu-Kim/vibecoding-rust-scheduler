import type { ViewMode } from '../types';

interface SidebarProps {
  currentView: ViewMode;
  onViewChange: (view: ViewMode) => void;
  selectedDate: string;
  onDateChange: (date: string) => void;
}

export function Sidebar({ currentView, onViewChange, selectedDate, onDateChange }: SidebarProps) {
  return (
    <div className="w-64 bg-white border-r border-gray-200 h-screen flex flex-col">
      {/* Header */}
      <div className="p-6 border-b border-gray-200">
        <h1 className="text-2xl font-bold text-primary-600">📅 Scheduler</h1>
        <p className="text-sm text-gray-500 mt-1">Psychology-Based</p>
      </div>

      {/* Date Picker */}
      <div className="p-4 border-b border-gray-200">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Selected Date
        </label>
        <input
          type="date"
          value={selectedDate}
          onChange={(e) => onDateChange(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent text-sm"
        />
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-4">
        <div className="space-y-2">
          <NavButton
            icon="📋"
            label="스케줄"
            active={currentView === 'schedule'}
            onClick={() => onViewChange('schedule')}
          />
          <NavButton
            icon="⏱️"
            label="타임라인"
            active={currentView === 'timeline'}
            onClick={() => onViewChange('timeline')}
          />
          <NavButton
            icon="📊"
            label="대시보드"
            active={currentView === 'dashboard'}
            onClick={() => onViewChange('dashboard')}
          />
        </div>
      </nav>

      {/* Footer */}
      <div className="p-4 border-t border-gray-200">
        <div className="text-xs text-gray-500">
          <p className="font-medium">v2.0.0 - GUI Edition</p>
          <p className="mt-1">12 Psychology Principles</p>
        </div>
      </div>
    </div>
  );
}

interface NavButtonProps {
  icon: string;
  label: string;
  active: boolean;
  onClick: () => void;
}

function NavButton({ icon, label, active, onClick }: NavButtonProps) {
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center space-x-3 px-4 py-3 rounded-lg transition-colors ${
        active
          ? 'bg-primary-50 text-primary-700 font-medium'
          : 'text-gray-700 hover:bg-gray-50'
      }`}
    >
      <span className="text-xl">{icon}</span>
      <span>{label}</span>
    </button>
  );
}
