import { Message } from '../types/room';
import { UserCircleIcon } from '@heroicons/react/24/solid';
import { formatTimestamp } from '../utils/time';
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { oneLight } from 'react-syntax-highlighter/dist/esm/styles/prism'
import { useState } from 'react';
import { ClipboardDocumentIcon, ShareIcon } from '@heroicons/react/24/outline';
import { CheckIcon } from '@heroicons/react/24/solid';

interface ChatMessageProps {
  message: Message;
  isCurrentUser: boolean;
}

const shortenAddress = (address: string) => {
  if (!address) return '';
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
};

export function ChatMessage({ message, isCurrentUser }: ChatMessageProps) {
  const [copied, setCopied] = useState(false);
  const isAI = message.message_type === 1; // Updated from messageType
  const timestamp = message.timestamp;
  const displayName = isAI ? 'AI Assistant' : shortenAddress(message.sender);

  const handleCopy = async () => {
    const shareText = `${message.content}\n\n${window.location.href}`;
    await navigator.clipboard.writeText(shareText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleShare = () => {
    const shareText = `${message.content}\n\n${window.location.href}`;
    const twitterUrl = `https://twitter.com/intent/tweet?text=${encodeURIComponent(shareText)}`;
    window.open(twitterUrl, '_blank');
  };

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
          <div className="relative group">
            <div
              className={`rounded-lg px-4 py-2 max-w-[80%] ${
                isCurrentUser
                  ? 'bg-blue-50 text-gray-900 border border-blue-100'  // Lighter blue for user messages
                  : isAI
                  ? 'bg-purple-50 border border-purple-100'            // Keep AI message style
                  : 'bg-gray-50 border border-gray-100'               // Lighter gray for other users
              }`}
            >
              <div className="flex justify-between items-start gap-4">
                <div className="text-sm leading-relaxed flex-1">
                  <ReactMarkdown 
                    remarkPlugins={[remarkGfm]}
                    className="prose prose-sm max-w-none"
                    components={{
                      pre: ({children}) => children,
                      code: ({node, inline, className, children, ...props}) => {
                        const match = /language-(\w+)/.exec(className || '')
                        const language = match ? match[1] : ''
                        
                        return !inline ? (
                          <div className="my-4">
                            <SyntaxHighlighter
                              language={language}
                              style={oneLight}
                              customStyle={{
                                backgroundColor: '#f8fafc',  // bg-slate-50
                                padding: '1rem',
                                borderRadius: '0.375rem',
                                border: '1px solid #e2e8f0',  // border-slate-200
                              }}
                            >
                              {String(children).replace(/\n$/, '')}
                            </SyntaxHighlighter>
                          </div>
                        ) : (
                          <code
                            className={`px-1.5 py-0.5 rounded ${
                              isCurrentUser 
                                ? 'bg-blue-100/50 text-blue-800' 
                                : 'bg-slate-100 text-slate-800'
                            }`}
                            {...props}
                          >
                            {children}
                          </code>
                        )
                      },
                    }}
                  >
                    {message.content}
                  </ReactMarkdown>
                </div>
                
                {/* Move action buttons inside message bubble */}
                <div className="flex flex-col gap-2 pt-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onClick={handleCopy}
                    className="p-1.5 rounded-full hover:bg-white/50 text-gray-500 hover:text-gray-700 transition-colors"
                    title="Copy message"
                  >
                    {copied ? (
                      <CheckIcon className="w-4 h-4 text-green-500" />
                    ) : (
                      <ClipboardDocumentIcon className="w-4 h-4" />
                    )}
                  </button>
                  <button
                    onClick={handleShare}
                    className="p-1.5 rounded-full hover:bg-white/50 text-gray-500 hover:text-gray-700 transition-colors"
                    title="Share on Twitter"
                  >
                    <ShareIcon className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
        {isCurrentUser && <div className="flex-shrink-0 w-8 h-8" />}
      </div>
    </div>
  );
}