// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { renderHook, waitFor, act } from '@testing-library/react'
import { createWalletProviderContextWrapper, registerMockWallet } from '../utils.js'
import { useAutoConnectWallet, useConnectWallet, useCurrentWallet } from '../../src/index.js'

function withResolvers<T = any>() {
  let resolve, reject
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, reject: reject!, resolve: resolve! }
}

describe('useAutoConnectWallet', () => {
  test('returns "disabled" when the auto-connect functionality is disabled', async () => {
    const wrapper = createWalletProviderContextWrapper()
    const { result } = renderHook(() => useAutoConnectWallet(), { wrapper })
    await waitFor(() => {
      expect(result.current).toBe('disabled')
    })
  })

  test(`returns "attempted" immediately when there is no last connected wallet`, async () => {
    const wrapper = createWalletProviderContextWrapper({ autoConnect: true })
    const { result } = renderHook(() => useAutoConnectWallet(), { wrapper })
    await waitFor(() => {
      expect(result.current).toBe('attempted')
    })
  })

  test('returns "attempted" when we have made a successful auto-connection attempt', async () => {
    const { mockWallet, unregister } = registerMockWallet()

    const wrapper = createWalletProviderContextWrapper({
      autoConnect: true,
    })
    const { result, unmount } = renderHook(() => useConnectWallet(), { wrapper })

    // Manually connect a wallet so we have a wallet to auto-connect to later.
    result.current.mutate({
      wallet: mockWallet,
    })

    await waitFor(() => expect(result.current.isSuccess).toBe(true))

    // Now unmount our component tree to simulate someone leaving the page.
    unmount()

    const { promise, resolve } = withResolvers()
    mockWallet.mocks.connect.mockImplementation(async () => {
      return promise
    })

    // Render our component tree again and auto-connect to our previously connected wallet account.
    const { result: updatedResult } = renderHook(
      () => ({ autoConnect: useAutoConnectWallet(), wallet: useCurrentWallet() }),
      { wrapper },
    )

    // Expect the initial state to be idle:
    expect(updatedResult.current.autoConnect).toBe('idle')

    // Wait for the status to flip to connecting:
    await waitFor(() => expect(updatedResult.current.wallet.isConnecting).toBe(true))
    // The state should still be idle while the connection is in progress:
    expect(updatedResult.current.autoConnect).toBe('idle')

    resolve({ accounts: mockWallet.getBitcoinAddress() })

    // Now that the connection has completed, the state should be "attempted":
    await waitFor(() => expect(updatedResult.current.autoConnect).toBe('attempted'))
    expect(updatedResult.current.wallet.isConnected).toBe(true)

    act(() => unregister())
  })
})
