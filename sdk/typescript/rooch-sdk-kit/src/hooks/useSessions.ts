// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochSessionStore } from './index'

/**
 * Retrieves the all session account
 */
export function useSession() {
  return useRoochSessionStore((state) => state.sessions)
}
