// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochSessionStore } from './index.js'

/**
 * Retrieves the all session account
 */
export function useSession() {
  return useRoochSessionStore((state) =>
    state.sessions.sort((a, b) => b.getCreateTime() - a.getCreateTime()),
  )
}
