export function TypingIndicator() {
  return (
    <div className="flex items-start gap-3 p-4 max-w-3xl mx-auto">
      <div className="flex-shrink-0 w-8 h-8">
        <div className="w-8 h-8 rounded-full bg-gradient-to-r from-purple-500 to-blue-500 flex items-center justify-center text-white text-sm font-bold">
          AI
        </div>
      </div>
      <div className="flex flex-col flex-1">
        <div className="flex items-center gap-2 text-xs text-gray-500 mb-1">
          <span className="font-medium">AI Assistant</span>
          <span>•</span>
          <span>Thinking...</span>
        </div>
        <div className="rounded-lg px-4 py-2 bg-purple-50 border border-purple-100">
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 bg-purple-400 rounded-full animate-bounce [animation-delay:-0.3s]"></div>
            <div className="w-2 h-2 bg-purple-400 rounded-full animate-bounce [animation-delay:-0.15s]"></div>
            <div className="w-2 h-2 bg-purple-400 rounded-full animate-bounce"></div>
          </div>
        </div>
      </div>
    </div>
  );
}