// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useSessionStore } from './useSessionsStore.js'

/**
 * Retrieves the all session account
 */
export function useSession(scope: string) {
  return useSessionStore((state) =>
    state.sessions.find((item) =>
      scope.includes('::')
        ? item.getScopes().find((_scope) => _scope === scope)
        : item.getScopes().find((_scope) => _scope.startsWith(scope)) !== undefined,
    ),
  )
}
