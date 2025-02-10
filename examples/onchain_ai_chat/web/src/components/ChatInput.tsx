import { useState, useRef, useEffect } from 'react';
import { PaperAirplaneIcon } from '@heroicons/react/24/solid';
import { XMarkIcon } from '@heroicons/react/24/outline';
import { SessionKeyGuard } from '@roochnetwork/rooch-sdk-kit';

interface ChatInputProps {
  onSend: (message: string) => Promise<void>;  // Change to return Promise
  placeholder?: string;
  disabled?: boolean;
  value?: string;
  onChange?: (value: string) => void;
}

export function ChatInput({ 
  onSend, 
  placeholder = "Type a message...", 
  disabled = false,
  value,
  onChange 
}: ChatInputProps) {
  const [localValue, setLocalValue] = useState(value || '');
  const [showWarning, setShowWarning] = useState(true);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [localValue]);

  useEffect(() => {
    if (value !== undefined) {
      setLocalValue(value);
    }
  }, [value]);

  const handleSubmit = async () => {
    const message = localValue.trim();
    if (message && !disabled) {
      try {
        await onSend(message);
        // Only clear input if message was sent successfully
        if (value === undefined) {
          setLocalValue('');
        }
      } catch (error) {
        // Keep the input value if sending failed
        console.error('Failed to send message:', error);
      }
    }
  };

  return (
    <div className="w-full flex flex-col items-center">
      {showWarning && (
        <div className="w-full max-w-3xl mb-2 px-4">
          <div className="relative text-sm text-amber-600 bg-amber-50 rounded-lg p-3 pr-10 border border-amber-200">
            <span className="font-medium">Note:</span> This is an on-chain AI chat. All messages are public and permanently stored on the blockchain. Please do not share any private or sensitive information.
            <button
              onClick={() => setShowWarning(false)}
              className="absolute top-2 right-2 p-1 text-amber-600 hover:text-amber-700 rounded-full hover:bg-amber-100 transition-colors"
              aria-label="Close warning"
            >
              <XMarkIcon className="h-5 w-5" />
            </button>
          </div>
        </div>
      )}
      <div className="relative w-full flex justify-center">
        <div className="relative w-full max-w-3xl">
            <textarea
              ref={textareaRef}
              value={localValue}
              onChange={(e) => {
                setLocalValue(e.target.value);
                onChange?.(e.target.value);
              }}
              placeholder={placeholder}
              disabled={disabled}
              className="w-full resize-none rounded-lg border border-gray-200 pr-12 py-3 px-4 
                focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-50
                min-h-[60px] max-h-[200px] overflow-y-auto"
              rows={3}
            />
            <SessionKeyGuard onClick={handleSubmit}>
              <button
                type="button"
                disabled={disabled || !localValue.trim()}
                className="absolute right-2 bottom-2 p-2 text-blue-600 hover:text-blue-700 
                  disabled:text-gray-400"
              >
                <PaperAirplaneIcon className="h-6 w-6" />
              </button>
            </SessionKeyGuard>
        </div>
      </div>
    </div>
  );
}