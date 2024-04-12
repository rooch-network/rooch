// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Network } from '@roochnetwork/rooch-sdk'
import { useRoochContextStore } from './index'

export function useCurrentNetwork(): Network {
  return useRoochContextStore((state) => state.currentNetwork)
}
