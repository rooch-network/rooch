// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochContext } from './index.js'
import { NetworkType } from '@roochnetwork/rooch-sdk'

export function useCurrentNetwork(): NetworkType {
  return useRoochContext().network
}
