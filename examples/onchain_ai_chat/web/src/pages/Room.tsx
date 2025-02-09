import { useState, useRef, useEffect, useMemo } from 'react'
import { useParams } from 'react-router-dom'
import { 
  useCurrentSession, 
  useRoochClient, 
  useRoochClientQuery,
  SessionKeyGuard
} from '@roochnetwork/rooch-sdk-kit'
import { Layout } from '../components/Layout'
import { ChatInput } from '../components/ChatInput'
import { ChatMessage } from '../components/ChatMessage'
import { useNetworkVariable } from '../networks'
import { Args, Transaction, bcs } from '@roochnetwork/rooch-sdk'
import { Message, MessageSchema} from '../types/room'

export function Room() {
  const { roomId } = useParams<{ roomId: string }>();
  const sessionKey = useCurrentSession()
  const client = useRoochClient()
  const [loading, setLoading] = useState(false)
  const packageId = useNetworkVariable('packageId')
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const PAGE_SIZE = 20;
  const [allMessages, setAllMessages] = useState<Message[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const loadMoreRef = useRef<HTMLDivElement>(null);
  
  // Query messages count
  const { data: messageCountResponse, refetch: refetchMessageCount } = useRoochClientQuery(
    'executeViewFunction',
    {
      target: `${packageId}::room::get_message_count`,
      args: [Args.objectId(roomId!)],
    },
    {
      enabled: !!roomId && !!client,
    }
  );

  // Update total count when messageCountResponse changes
  useEffect(() => {
    if (messageCountResponse?.return_values?.[0]?.decoded_value) {
      setTotalCount(parseInt(messageCountResponse.return_values[0].decoded_value));
    }
  }, [messageCountResponse]);

  // Query messages using pagination - Fix pagination logic
  const { data: messagesResponse, refetch: refetchMessages } = useRoochClientQuery(
    'executeViewFunction',
    {
      target: `${packageId}::room::get_messages_paginated`,
      args: [
        Args.objectId(roomId!),
        Args.u64(Math.max(0, totalCount - (page + 1) * PAGE_SIZE)), // ensure non-negative start index
        Args.u64(Math.min(PAGE_SIZE, totalCount - page * PAGE_SIZE)), // ensure valid limit
      ],
    },
    {
      enabled: !!roomId && !!client && totalCount > 0,
    }
  );

  const deserializeMessages = (hexValue: string) => {
    try {
      const cleanHexValue = hexValue.startsWith('0x') ? hexValue.slice(2) : hexValue;
      const bytes = new Uint8Array(
        cleanHexValue.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
      );
      
      const parsedMessages = bcs.vector(MessageSchema).parse(bytes);
      return parsedMessages.map((message: any) => ({
        sender: message.sender,
        content: message.content,
        timestamp: message.timestamp,
        message_type: message.message_type, // Updated to match Move field name
      }));
    } catch (error) {
      console.error('BCS deserialization error:', error);
      return [];
    }
  };

  // Reset state when roomId changes
  useEffect(() => {
    setAllMessages([]);
    setPage(0);
    setHasMore(true);
    setTotalCount(0);
  }, [roomId]);

  // Fix message handling
  useEffect(() => {
    if (messagesResponse?.return_values?.[0]?.value?.value) {
      const newMessages = deserializeMessages(messagesResponse.return_values[0].value.value);
      setAllMessages(prev => {
        // If it's a new page (not page 0), append messages
        // If it's page 0 (either initial load or new message), replace all messages
        const baseMessages = page === 0 ? [] : prev;
        
        // Create a map of existing messages
        const existingMessages = new Map(baseMessages.map(msg => [
          `${msg.sender}-${msg.timestamp}`,
          msg
        ]));
        
        // Add new messages to map
        newMessages.forEach(msg => {
          existingMessages.set(`${msg.sender}-${msg.timestamp}`, msg);
        });
        
        const sortedMessages = Array.from(existingMessages.values())
          .sort((a, b) => parseInt(a.timestamp) - parseInt(b.timestamp));

        // Only scroll if we're not loading more messages
        if (page === 0) {
          setTimeout(scrollToBottom, 100);
        } else {
        // Scroll to the first new message when loading more
          setTimeout(() => {
            loadMoreRef.current?.scrollIntoView({ behavior: 'smooth' });
          }, 100);
        }

        // Update hasMore based on actual message count and current page
        const currentMessageCount = existingMessages.size;
        setHasMore(currentMessageCount < totalCount);

        return sortedMessages;
      });
    }
  }, [messagesResponse, totalCount, page, roomId]); // Add roomId to dependencies

  const loadMoreMessages = () => {
    if (!loading && hasMore) {
      setPage(prev => prev + 1);
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  const handleSendMessage = async (message: string) => {
    if (loading || !roomId) {
      return;
    }

    setLoading(true);

    try {
      const tx = new Transaction();
      tx.callFunction({
        target: `${packageId}::room::send_message_entry`,
        args: [Args.objectId(roomId), Args.string(message)],
      });

      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: sessionKey!,
      });

      if (result.execution_info.status.type !== 'executed') {
        // Throw error instead of just logging
        throw new Error('Failed to send message: Transaction not executed');
      }

      // Reset to first page and refetch
      setPage(0);
      await Promise.all([
        refetchMessages(),
        refetchMessageCount(),
      ]);
      
      // Delay scroll to bottom to ensure new message is rendered
      setTimeout(scrollToBottom, 100);
    } catch (error) {
      // Re-throw the error to trigger ErrorGuard
      console.error('Failed to send message:', error);
      throw error;
    } finally {
      setLoading(false);
    }
  };

  // Update render logic - Remove reverse()
  return (
    <Layout showRoomList>
      <div className="flex-1 min-h-0 flex flex-col">
        {totalCount > 0 && hasMore && (
          <button
            onClick={loadMoreMessages}
            disabled={loading}
            className="text-blue-500 hover:text-blue-700 p-4 text-center disabled:text-gray-400"
          >
            {loading ? 'Loading...' : 'Load More Messages'}
          </button>
        )}
        <div className="flex-1 overflow-y-auto px-4 py-2">
          <div className="space-y-4">
            <div ref={loadMoreRef} /> {/* Add ref for load more scroll position */}
            {allMessages.map((message, index) => (
              <ChatMessage
                key={`${message.sender}-${message.timestamp}-${index}`}
                message={message}
                isCurrentUser={message.sender === sessionKey?.roochAddress.toHexAddress()}
              />
            ))}
            <div ref={messagesEndRef} />
          </div>
        </div>
        <div className="flex-none p-4 border-t bg-white">
          <ChatInput 
            onSend={handleSendMessage}
            disabled={loading}
            placeholder="Type a message..."
          />
        </div>
      </div>
    </Layout>
  );
}