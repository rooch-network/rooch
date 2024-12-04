// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useWalletStore } from './useWalletStore'

export function useConnectionStatus() {
  return useWalletStore((state) => state.connectionStatus)
}
