import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useRoochClient } from './useRoochClient.js'
import {
  ErrorValidateSessionIsExpired,
  JsonRpcError,
  Secp256k1Keypair,
  Session,
  Transaction,
} from '@roochnetwork/rooch-sdk'
import { useSessionStore } from '../hooks/useSessionsStore.js'

vi.mock('../hooks/useSessionsStore')

describe('useRoochClient', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.clearAllMocks()
  })

  // issue-2481
  it('should call store `removeSession` when client receives session expired error', async () => {
    const SDK = await import('@roochnetwork/rooch-sdk')

    SDK.RoochHTTPTransport.prototype.request = vi
      .fn()
      .mockRejectedValue(
        new JsonRpcError('[test] session expired', ErrorValidateSessionIsExpired),
      )

    const mockSigner = Secp256k1Keypair.generate()
    const mockSession = { authKey: 'test-session-1' } as unknown as Session

    const networks = { test: { url: 'https://test.com' } }

    const mockRemoveSession = vi.fn()
    vi.mocked(useSessionStore).mockImplementation((selector) => {
      const mockStore = {
        currentSession: mockSession,
        sessions: [mockSession],
        addSession: vi.fn(),
        setCurrentSession: vi.fn(),
        removeSession: mockRemoveSession,
      }

      if (typeof selector === 'function') {
        return selector(mockStore)
      }
      return mockStore
    })

    const { result } = renderHook(() => useRoochClient({ currentNetwork: 'test', networks }))

    await expect(async () => {
      await result.current.signAndExecuteTransaction({
        transaction: new Transaction(),
        signer: mockSigner,
      })
    }).rejects.toThrowError('[test] session expired')

    expect(mockRemoveSession).toBeCalledWith(mockSession)
  })
})
