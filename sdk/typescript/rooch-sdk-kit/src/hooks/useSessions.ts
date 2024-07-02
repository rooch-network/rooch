// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useSessionStore } from './useSessionsStore.js'

/**
 * Retrieves the all session account
 */
export function useSession() {
  return useSessionStore((state) =>
    state.sessions.sort((a, b) => b.getCreateTime() - a.getCreateTime()),
  )
}
