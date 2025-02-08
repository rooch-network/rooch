import { Message } from '../types/room';
import { UserCircleIcon } from '@heroicons/react/24/solid';

interface ChatMessageProps {
  message: Message;
  isCurrentUser: boolean;
}

export function ChatMessage({ message, isCurrentUser }: ChatMessageProps) {
  console.log('Message:', message);
  const isAI = message.message_type === 1; // 修改为 message_type 来匹配后端返回的字段
  const timestamp = parseInt(message.timestamp); // 确保 timestamp 是数字

  return (
    <div className={`flex gap-3 ${isCurrentUser ? 'justify-end' : ''}`}>
      {!isCurrentUser && (
        <div className="flex-shrink-0 w-8 h-8">
          {isAI ? (
            <div className="w-8 h-8 rounded-full bg-gradient-to-r from-purple-500 to-blue-500 flex items-center justify-center text-white text-sm font-bold">
              AI
            </div>
          ) : (
            <UserCircleIcon className="w-8 h-8 text-gray-400" />
          )}
        </div>
      )}
      <div className={`flex flex-col ${isCurrentUser ? 'items-end' : 'items-start'}`}>
        <div className="text-xs text-gray-500 mb-1">
          {!isNaN(timestamp) && new Date(timestamp * 1000).toLocaleTimeString()}
        </div>
        <div
          className={`rounded-lg px-4 py-2 max-w-2xl whitespace-pre-wrap break-words ${
            isCurrentUser
              ? 'bg-blue-500 text-white'
              : isAI
              ? 'bg-purple-50 border border-purple-100'
              : 'bg-gray-100'
          }`}
        >
          {message.content}
        </div>
      </div>
    </div>
  );
}