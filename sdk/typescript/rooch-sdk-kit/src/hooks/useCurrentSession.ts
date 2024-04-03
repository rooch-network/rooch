// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IAccount } from '@roochnetwork/rooch-sdk'
import { useRoochContext } from './index'

/**
 * Retrieves the session account that is currently selected, if one exists.
 */
export function useCurrentSession(): IAccount | null {
  return useRoochContext().currentSession
}
