// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochClient } from '@roochnetwork/rooch-sdk'
import { useRoochContext } from './index'

export function useRoochClient(): RoochClient {
  return useRoochContext().client
}
