// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useWalletStore } from './useWalletStore.js'

/**
 * Retrieves all wallets
 */
export function useCurrentAddress() {
  return useWalletStore((state) => state.currentAddress)
}
