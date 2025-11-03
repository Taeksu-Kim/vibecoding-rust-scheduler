import ReactMarkdown from 'react-markdown';

interface ClaudeModalProps {
  response: string;
  isLoading: boolean;
  onClose: () => void;
  title?: string;
}

export function ClaudeModal({ response, isLoading, onClose, title = 'Claude 스케줄 평가' }: ClaudeModalProps) {
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-3xl w-full max-h-[90vh] flex flex-col">
        <div className="p-6 border-b border-gray-200 flex-shrink-0 flex justify-between items-center">
          <div className="flex items-center space-x-2">
            <span className="text-2xl">🤖</span>
            <h3 className="text-xl font-bold text-gray-800">{title}</h3>
          </div>
          {!isLoading && (
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 text-2xl leading-none"
            >
              ×
            </button>
          )}
        </div>

        <div className="p-6 overflow-y-auto flex-1">
          {isLoading ? (
            <div className="flex flex-col items-center justify-center py-12 space-y-4">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
              <p className="text-gray-600">Claude가 분석하고 있습니다...</p>
              <p className="text-sm text-gray-500">약 10-30초 정도 소요될 수 있습니다</p>
            </div>
          ) : (
            <div className="prose prose-sm max-w-none prose-headings:text-gray-800 prose-h2:text-lg prose-h2:font-bold prose-h2:mt-4 prose-h2:mb-2 prose-h3:text-base prose-h3:font-semibold prose-h3:mt-3 prose-h3:mb-1 prose-p:text-gray-700 prose-p:leading-relaxed prose-ul:list-disc prose-ul:ml-4 prose-li:text-gray-700 prose-strong:text-gray-900 prose-strong:font-semibold prose-code:bg-gray-100 prose-code:px-1 prose-code:rounded prose-code:text-sm">
              <ReactMarkdown>{response}</ReactMarkdown>
            </div>
          )}
        </div>

        {!isLoading && (
          <div className="p-6 border-t border-gray-200 flex-shrink-0 flex justify-end">
            <button
              onClick={onClose}
              className="px-6 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
            >
              닫기
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
