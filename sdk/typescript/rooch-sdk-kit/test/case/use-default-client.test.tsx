import { useEffect } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import {
  ErrorValidateSessionIsExpired,
  JsonRpcError,
  RoochClient,
  Session,
  Transaction,
} from '@roochnetwork/rooch-sdk'
import { useDefaultClient } from '../../src/provider/useDefaultClient.js'
import { createWalletProviderContextWrapper, registerMockWallet } from '../utils.js'
import {
  useConnectWallet,
  useCreateSessionKey,
  useCurrentSession,
  useSession,
} from '../../src/index.js'

describe('useDefaultClient', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.clearAllMocks()
  })

  // issue-2481
  it('should remove expired session from storage when receives expired error', async () => {
    const { mockWallet } = registerMockWallet()

    const mockClient = new RoochClient({ url: 'http://localhost:6767' })
    mockClient.signAndExecuteTransaction = vi.fn().mockResolvedValue({
      execution_info: { status: { type: 'executed' } },
    })

    function useTestHook() {
      const { mutateAsync: connectWallet } = useConnectWallet()
      const { mutateAsync: createSessionKey } = useCreateSessionKey()
      const sessions = useSession()
      const currentSession = useCurrentSession()
      const networks = { test: { url: 'http://localhost:6767' } }
      const defaultClient = useDefaultClient({ currentNetwork: 'test', networks })

      useEffect(() => {
        async function createSession() {
          await createSessionKey({
            appName: 'app-name',
            appUrl: 'app-url',
            scopes: ['0x1::*::*'],
          })
        }

        async function connectAndCreateSession() {
          await connectWallet({ wallet: mockWallet })
          // create two sessions
          await createSession()
          await createSession()
        }

        connectAndCreateSession()
      }, [])

      const triggerSessionExpiredError = async () => {
        const SDK = await import('@roochnetwork/rooch-sdk')
        SDK.RoochHTTPTransport.prototype.request = vi
          .fn()
          .mockRejectedValue(
            new JsonRpcError('[test] session expired', ErrorValidateSessionIsExpired),
          )

        return defaultClient.signAndExecuteTransaction({
          transaction: new Transaction(),
          signer: currentSession!,
        })
      }

      return { currentSession, sessions, triggerSessionExpiredError }
    }

    const wrapper = createWalletProviderContextWrapper({}, mockClient)
    const { result } = renderHook(() => useTestHook(), { wrapper })

    await waitFor(() => {
      expect(result.current.sessions).toHaveLength(2)
      expect(result.current.currentSession).toBeDefined()
    })

    const cachedCurrentSession = result.current.currentSession
    const getMatchedSessionByAuthKey = (s: Session) => {
      return cachedCurrentSession!.getAuthKey() === s.getAuthKey()
    }

    await expect(result.current.triggerSessionExpiredError).rejects.toThrow(
      '[test] session expired',
    )
    expect(result.current.sessions).toHaveLength(1)
    expect(result.current.sessions.find(getMatchedSessionByAuthKey)).toBeUndefined()
  })
})
