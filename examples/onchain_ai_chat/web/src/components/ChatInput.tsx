import { useState, useRef, useEffect } from 'react';
import { PaperAirplaneIcon } from '@heroicons/react/24/solid';
import { SessionKeyGuard } from '@roochnetwork/rooch-sdk-kit';

interface ChatInputProps {
  onSend: (message: string) => void;
  placeholder?: string;
  disabled?: boolean;
}

export function ChatInput({ onSend, placeholder = "Type a message...", disabled = false }: ChatInputProps) {
  const [message, setMessage] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [message]);

  const handleSubmit = () => {
    if (message.trim() && !disabled) {
      onSend(message.trim());
      setMessage('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  return (
    <div className="relative w-[600px]">
      <textarea
        ref={textareaRef}
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        className="w-full resize-none rounded-lg border border-gray-200 pr-12 py-3 px-4 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-50"
        rows={1}
      />
      <SessionKeyGuard onClick={handleSubmit}>
          <button
            type="button"
            disabled={!message.trim() || disabled}
            className="absolute right-2 bottom-2 p-2 text-blue-600 hover:text-blue-700 disabled:text-gray-400"
          >
            <PaperAirplaneIcon className="h-6 w-6" />
          </button>
      </SessionKeyGuard>
    </div>
  );
}