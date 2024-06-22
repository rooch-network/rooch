// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createWalletProviderContextWrapper } from '../utils.js'
import { renderHook } from '@testing-library/react'
import { useAutoConnectWallet } from '../../src/index.js'

describe('useAutoConnectWallet', () => {
  test('returns "disabled" when the auto-connect functionality is disabled', async () => {
    const wrapper = createWalletProviderContextWrapper()
    const { result } = renderHook(() => useAutoConnectWallet(), { wrapper })
    expect(result.current).toBe('disabled')
  })
})
