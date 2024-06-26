// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochContext } from './index.js'

export function useCurrentNetwork(): string {
  return useRoochContext().network
}
