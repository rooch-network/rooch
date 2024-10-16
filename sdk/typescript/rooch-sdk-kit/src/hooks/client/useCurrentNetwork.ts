// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochContext } from './index.js'
import { NetWorkType } from '@roochnetwork/rooch-sdk'

export function useCurrentNetwork(): NetWorkType {
  return useRoochContext().network
}
