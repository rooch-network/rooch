import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Layout } from '../components/Layout'
import { ChatInput } from '../components/ChatInput'
import { Title } from '../components/Title';
import { RoomListContainer } from '../containers/RoomListContainer'
import { 
  useCurrentSession, 
  useRoochClient,
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
        target: `${packageId}::room::create_ai_room_with_message_entry`,
        args: [
          Args.string("new_chat"),  // title
          Args.bool(true),          // is_public
          Args.string(message),     // first_message
        ],
      })

      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: sessionKey,
      })

      if (result?.execution_info.status.type !== 'executed') {
        throw new Error(`Failed to create room and send message, ${JSON.stringify(result.execution_info)}`);
      }

      // Find the Room object from changeset
      const roomChange = result.output?.changeset.changes.find(
        change => change.metadata.object_type.endsWith('::room::Room')
      )

      if (!roomChange?.metadata.id) {
        throw new Error('Failed to get room ID from transaction result')
      }

      const roomId = roomChange.metadata.id
      console.log('Created room with ID:', roomId)
      navigate(`/chat/${roomId}`)
    } catch (error) {
      console.error('Failed to create chat:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
      <Title title="OnChain AI Chat" />
      <Layout>
        <div className="flex h-full">
          {/* Sidebar with room list */}
          <div className="w-64 border-r border-gray-200 overflow-y-auto">
            <div className="p-4">
              <h2 className="text-lg font-semibold text-gray-700 mb-4">Recent Chats</h2>
              <RoomListContainer />
            </div>
          </div>

          {/* Main content */}
          <div className="flex-1 flex flex-col items-center justify-center p-4">
            <div className="w-full max-w-2xl space-y-6">
              <div className="text-center">
                <h1 className="text-4xl font-bold text-gray-900 mb-4">
                  OnChain AI Chat
                </h1>
                <p className="text-lg text-gray-600">
                  Start a public AI conversation that everyone can join
                </p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="p-4 rounded-lg border border-gray-200 hover:border-gray-300">
                  <h3 className="font-medium mb-2">Let's explore blockchain together</h3>
                  <p className="text-sm text-gray-500">"Explain how Move differs from Solidity in handling assets?"</p>
                </div>
                <div className="p-4 rounded-lg border border-gray-200 hover:border-gray-300">
                  <h3 className="font-medium mb-2">Start a community discussion</h3>
                  <p className="text-sm text-gray-500">"What are the key considerations for designing a DeFi protocol?"</p>
                </div>
                <div className="p-4 rounded-lg border border-gray-200 hover:border-gray-300">
                  <h3 className="font-medium mb-2">Learn with others</h3>
                  <p className="text-sm text-gray-500">"Help me understand zero-knowledge proofs with examples"</p>
                </div>
                <div className="p-4 rounded-lg border border-gray-200 hover:border-gray-300">
                  <h3 className="font-medium mb-2">Share knowledge</h3>
                  <p className="text-sm text-gray-500">"What's the future of decentralized identity?"</p>
                </div>
              </div>

              <ChatInput
                onSend={handleCreateRoom}
                placeholder="Start a public discussion..."
                disabled={loading}
              />
            </div>
          </div>
        </div>
      </Layout>
    </>
  )
}