import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Layout } from '../components/Layout'
import { ChatInput } from '../components/ChatInput'
import { 
  useCurrentSession, 
  useRoochClient,
  SessionKeyGuard 
} from '@roochnetwork/rooch-sdk-kit'
import { useNetworkVariable } from '../networks'
import { Args, Transaction } from '@roochnetwork/rooch-sdk'

export function Home() {
  const navigate = useNavigate()
  const sessionKey = useCurrentSession()
  const client = useRoochClient()
  const packageId = useNetworkVariable('packageId')
  const [loading, setLoading] = useState(false)

  const handleCreateRoom = async (message: string) => {
  console.log('Creating room with message:', message, client, sessionKey, loading)

    if (!client || !sessionKey || loading) return
    setLoading(true)

    try {
      const tx = new Transaction()
      tx.callFunction({
        target: `${packageId}::room::create_ai_room_entry`,
        args: [
          Args.string("new_chat"), 
          Args.bool(          true), // public room
        ],
      })

      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: sessionKey,
      })

      if (result?.execution_info.status.type !== 'executed') {
        throw new Error('Create room failed')
      }

      // Find the Room object from changeset
      const roomChange = result.output?.changeset.changes.find(
        change => change.metadata.object_type.endsWith('::room::Room')
      )

      if (!roomChange?.metadata.id) {
        throw new Error('Failed to get room ID from transaction result')
      }

      const roomId = roomChange.metadata.id
      console.log('Created room:', roomId)

      // Send initial message
      const messageTx = new Transaction()
      messageTx.callFunction({
        target: `${packageId}::room::send_message_entry`,
        args: [Args.objectId(roomId), Args.string(message)],
      })

      const messageResult = await client.signAndExecuteTransaction({
        transaction: messageTx,
        signer: sessionKey,
      });

      if (messageResult?.execution_info.status.type !== 'executed') {
        throw new Error('Failed to send message');
      }

      navigate(`/chat/${roomId}`); // Navigate to the new room
    } catch (error) {
      console.error('Failed to create chat:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Layout>
      <div className="flex flex-col items-center justify-center h-full p-4">
        <div className="w-full max-w-2xl">
          <h2 className="text-3xl font-bold text-center mb-8 text-gray-900">
            How can I help you today?
          </h2>
          <div className="w-full">
            <ChatInput
              onSend={handleCreateRoom}
              placeholder="Send a message to start a new chat..."
              disabled={loading}
            />
          </div>
        </div>
      </div>
    </Layout>
  );
}