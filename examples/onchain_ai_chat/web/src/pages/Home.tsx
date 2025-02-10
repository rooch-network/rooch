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
import { ErrorToast } from '../components/ErrorToast'
import { getErrorMessage } from '../utils/errors'

export function Home() {
  const navigate = useNavigate()
  const sessionKey = useCurrentSession()
  const client = useRoochClient()
  const packageId = useNetworkVariable('packageId')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [inputValue, setInputValue] = useState('');

  const exampleQuestions = [
    {
      title: "Let's explore blockchain together",
      question: "Explain how Move differs from Solidity in handling assets?"
    },
    {
      title: "Start a community discussion",
      question: "What are the key considerations for designing a DeFi protocol?"
    },
    {
      title: "Learn with others",
      question: "Help me understand zero-knowledge proofs with examples"
    },
    {
      title: "Share knowledge",
      question: "What's the future of decentralized identity?"
    }
  ];

  const handleCreateRoom = async (message: string) => {
  console.log('Creating room with message:', message, client, sessionKey, loading)

    if (!client || !sessionKey || loading) return
    setLoading(true)
    setError(null)

    try {
      const tx = new Transaction()
      tx.callFunction({
        target: `${packageId}::room::create_ai_room_with_message_entry`,
        args: [
          Args.string("new_chat"),
          Args.bool(true),
          Args.string(message),
        ],
      })

      const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: sessionKey,
      })

      if (result?.execution_info.status.type !== 'executed') {
        console.error('Failed to send message:', result.execution_info);
        throw new Error(`Failed to create room and send message: Transaction not executed. status: ${JSON.stringify(result.execution_info.status)}`);
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
      const errorMessage = getErrorMessage(error)
      setError(errorMessage)
      // Throw error to prevent ChatInput from clearing
      throw error
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
                {exampleQuestions.map((item, index) => (
                  <button
                    key={index}
                    onClick={() => setInputValue(item.question)}
                    className="text-left p-4 rounded-lg border border-gray-200 hover:border-gray-300 hover:bg-gray-50 transition-colors"
                  >
                    <h3 className="font-medium mb-2">{item.title}</h3>
                    <p className="text-sm text-gray-500">{item.question}</p>
                  </button>
                ))}
              </div>

              <ChatInput
                onSend={handleCreateRoom}
                placeholder="Start a public discussion..."
                disabled={loading}
                value={inputValue}
                onChange={setInputValue}
              />
            </div>
          </div>
        </div>
      </Layout>
      {error && (
        <ErrorToast 
          message={error}
          onClose={() => setError(null)}
        />
      )}
    </>
  );
}