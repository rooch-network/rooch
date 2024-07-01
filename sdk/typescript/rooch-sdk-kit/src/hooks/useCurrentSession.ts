// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useSessionStore } from './useSessionsStore.js'

/**
 * Retrieves the session account that is currently selected, if one exists.
 */
export function useCurrentSession() {
  return useSessionStore((state) => state.currentSession)
}
