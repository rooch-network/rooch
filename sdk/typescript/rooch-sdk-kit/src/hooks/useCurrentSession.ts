// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochSessionStore } from './index.js'

/**
 * Retrieves the session account that is currently selected, if one exists.
 */
export function useCurrentSession() {
  return useRoochSessionStore((state) => state.currentSession)
}
