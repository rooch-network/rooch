import { Message } from '../types/room';
import { UserCircleIcon } from '@heroicons/react/24/solid';
import { formatTimestamp } from '../utils/time';
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

interface ChatMessageProps {
  message: Message;
  isCurrentUser: boolean;
}

const shortenAddress = (address: string) => {
  if (!address) return '';
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
};

export function ChatMessage({ message, isCurrentUser }: ChatMessageProps) {
  const isAI = message.message_type === 1; // Updated from messageType
  const timestamp = message.timestamp;
  const displayName = isAI ? 'AI Assistant' : shortenAddress(message.sender);

  return (
    <div className="flex justify-center w-full">
      <div className="w-full max-w-3xl flex gap-3">
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
        <div className={`flex flex-col flex-1 ${isCurrentUser ? 'items-end' : 'items-start'}`}>
          <div className="flex items-center gap-2 text-xs text-gray-500 mb-1">
            <span className="font-medium">
              {isCurrentUser ? 'You' : displayName}
            </span>
            <span>•</span>
            <span>
            {formatTimestamp(message.timestamp)}
            </span>
          </div>
          <div
            className={`rounded-lg px-4 py-2 max-w-[80%] ${
              isCurrentUser
                ? 'bg-blue-500 text-white'
                : isAI
                ? 'bg-purple-50 border border-purple-100'
                : 'bg-gray-100'
            }`}
          >
            <div className="text-sm">
              <ReactMarkdown 
                remarkPlugins={[remarkGfm]}
                components={{
                  pre: ({children}) => (
                    <pre className="overflow-x-auto bg-gray-50 rounded-md p-2 my-2">
                      {children}
                    </pre>
                  ),
                  code: ({node, inline, className, children, ...props}) => (
                    <code
                      className={`${inline 
                        ? 'px-1 py-0.5 rounded bg-opacity-20' 
                        : ''} 
                        ${isCurrentUser 
                          ? inline ? 'bg-white' : ''
                          : inline ? 'bg-gray-200' : ''}
                      `}
                      {...props}
                    >
                      {children}
                    </code>
                  ),
                }}
              >
                {message.content}
              </ReactMarkdown>
            </div>
          </div>
        </div>
        {isCurrentUser && <div className="flex-shrink-0 w-8 h-8" />}
      </div>
    </div>
  );
}