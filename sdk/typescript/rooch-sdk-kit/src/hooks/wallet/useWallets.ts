// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useWalletStore } from './index'

/**
 * Retrieves all wallets
 */
export function useSupportWallets() {
  return useWalletStore((state) => state.supportWallets)
}
