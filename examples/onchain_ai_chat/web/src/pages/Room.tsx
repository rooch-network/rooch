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
  
  // Query messages count - Always enabled when we have roomId and client
  const { data: messageCountResponse, refetch: refetchMessageCount } = useRoochClientQuery(
    'executeViewFunction',
    {
      target: `${packageId}::room::get_message_count`,
      args: [Args.objectId(roomId!)],
    },
    {
      enabled: !!roomId && !!client,
      refetchOnMount: true,
      refetchInterval: 2000, // Refresh every 2 seconds
    }
  );

  // Update message count effect to be independent
  useEffect(() => {
    if (messageCountResponse?.return_values?.[0]?.decoded_value) {
      const newCount = parseInt(messageCountResponse.return_values[0].decoded_value);
      setTotalCount(newCount);
    }
  }, [messageCountResponse]);

  // Update messages query to use correct pagination logic
  const { data: messagesResponse, refetch: refetchMessages } = useRoochClientQuery(
    'executeViewFunction',
    {
      target: `${packageId}::room::get_messages_paginated`,
      args: [
        Args.objectId(roomId!),
        // Start from the latest message and go backwards
        Args.u64(Math.max(0, totalCount - ((page + 1) * PAGE_SIZE))),
        // Calculate correct page size
        Args.u64(Math.min(
          PAGE_SIZE,
          // For the last page, we need to handle partial page
          totalCount - Math.max(0, totalCount - ((page + 1) * PAGE_SIZE))
        )),
      ],
    },
    {
      enabled: !!roomId && !!client && totalCount >= 0,
      refetchOnMount: true,
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
        id: message.id,
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

  // Reset state when room changes
  useEffect(() => {
    if (roomId) {
      // Clear messages immediately
      setAllMessages([]);
      setPage(0);
      setHasMore(false);
      
      // Fetch new room data
      Promise.all([
        refetchMessageCount(),
        refetchMessages()
      ]).catch(console.error);
    }
  }, [roomId, refetchMessageCount, refetchMessages]);

  // Handle messages update with better error handling
  useEffect(() => {
    if (messagesResponse?.return_values?.[0]?.value?.value) {
      try {
        const newMessages = deserializeMessages(messagesResponse.return_values[0].value.value);
        console.log('newMessages:', newMessages);
        setAllMessages(prev => {
          const baseMessages = page === 0 ? [] : prev;
          // Use message id as Map key instead of sender-timestamp
          const messageMap = new Map(baseMessages.map(msg => [msg.id, msg]));

          newMessages.forEach(msg => {
            messageMap.set(msg.id, msg);
          });

          const sortedMessages = Array.from(messageMap.values())
            // Sort by timestamp first, then by id for same timestamps
            .sort((a, b) => {
              const timeDiff = parseInt(a.timestamp) - parseInt(b.timestamp);
              return timeDiff === 0 ? a.id - b.id : timeDiff;
            });

          setHasMore(messageMap.size < totalCount);

          console.log('sortedMessages:', sortedMessages);
          return sortedMessages;
        });
      } catch (error) {
        console.error('Failed to process messages:', error);
      }
    }
  }, [messagesResponse, totalCount, page]);

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
      //5 RGas
      tx.setMaxGas(5_00000000);
      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: sessionKey!,
      });

      if (result.execution_info.status.type !== 'executed') {
        // Throw error instead of just logging
        throw new Error(`Failed to send message: Transaction not executed. Details: ${JSON.stringify(result.execution_info)}`);
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
      //throw error;
    } finally {
      setLoading(false);
    }
  };

  // Add loading state display
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
            <div ref={loadMoreRef} />
            {allMessages.length === 0 && !loading ? (
              <div className="flex items-center justify-center h-full text-gray-500">
                No messages yet
              </div>
            ) : (
              allMessages.map((message, index) => (
                <ChatMessage
                  key={`${message.sender}-${message.timestamp}-${index}`}
                  message={message}
                  isCurrentUser={message.sender === sessionKey?.roochAddress.toHexAddress()}
                />
              ))
            )}
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