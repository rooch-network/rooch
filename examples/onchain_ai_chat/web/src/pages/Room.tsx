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
import { Message, MessageSchema, transformMessage } from '../types/room'

export function Room() {
  const { roomId } = useParams<{ roomId: string }>()
  const sessionKey = useCurrentSession()
  const client = useRoochClient()
  const [loading, setLoading] = useState(false)
  const packageId = useNetworkVariable('packageId')
  const messagesEndRef = useRef<HTMLDivElement>(null)

  // Query messages using useRoochClientQuery
  const { data: messagesResponse, refetch: refetchMessages } = useRoochClientQuery(
    'executeViewFunction',
    {
      target: `${packageId}::room::get_messages`,
      args: [Args.objectId(roomId!)],
    },
    {
      enabled: !!roomId && !!client,
    }
  )

  const deserializeMessages = (hexValue: string) => {
    try {
      const cleanHexValue = hexValue.startsWith('0x') ? hexValue.slice(2) : hexValue;
      // Convert hex string to Uint8Array
      const bytes = new Uint8Array(
        cleanHexValue.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
      );
      
      const parsedMessages = bcs.vector(MessageSchema).parse(bytes);
      return parsedMessages.map((message: any) => ({
        sender: message.sender,
        content: message.content,
        timestamp: message.timestamp,
        messageType: message.messageType,
      }));
    } catch (error) {
      console.error('BCS deserialization error:', error);
      return [];
    }
  };

  // Decode messages using BCS
  const messages: Message[] = useMemo(() => {
    if (!messagesResponse?.return_values?.[0]?.value?.value) {
      return [];
    }
    return deserializeMessages(messagesResponse?.return_values?.[0]?.value?.value);
  }, [messagesResponse])

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    if (messages.length > 0) {
      console.log('Messages:', messages)
      scrollToBottom()
    }
  }, [messages])

  const handleSendMessage = async (message: string) => {
    if (loading || !roomId) {
      return
    }

    setLoading(true)

    const tx = new Transaction()
    tx.callFunction({
      target: `${packageId}::room::send_message_entry`,
      args: [Args.objectId(roomId), Args.string(message)],
    })

    try {
      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: sessionKey!,
      })

      if (result.execution_info.status.type !== 'executed') {
        console.error('Send message failed')
        return
      }

      await refetchMessages()
      scrollToBottom()
    } catch (error) {
      console.error('Failed to send message:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Layout showRoomList>
      <div className="flex h-full flex-col">
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {messages.map((message, index) => (
            <ChatMessage
              key={`${message.sender}-${message.timestamp}-${index}`}
              message={message}
              isCurrentUser={message.sender === sessionKey?.address}
            />
          ))}
          <div ref={messagesEndRef} />
        </div>
        <div className="p-4 border-t">
          <SessionKeyGuard onClick={handleSendMessage}>
            <ChatInput 
              onSend={handleSendMessage}
              disabled={loading}
              placeholder="Type a message..."
            />
          </SessionKeyGuard>
        </div>
      </div>
    </Layout>
  )
}