// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IAccount } from '@roochnetwork/rooch-sdk'
import { useWalletStore } from './useWalletStore.js'

/**
 * Retrieves the session account that is currently selected, if one exists.
 */
export function useCurrentSessionAccount(): IAccount | null {
  return useWalletStore((state) => state.sessionAccount)
}
