// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochContextStore } from './index'

export function useCurrentNetwork(): string {
  return useRoochContextStore((state) => state.currentNetwork)
}
