// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'
import { useStore } from 'zustand'
import type { ClientContextStoreState } from '../../clientStore'

import { RoochClientContext } from '../../provider'

export function useRoochContextStore<T>(selector: (state: ClientContextStoreState) => T): T {
  const store = useContext(RoochClientContext)
  if (!store) {
    throw new Error(
      'Could not find RoochClientContext. Ensure that you have set up the RoochClientProvider.',
    )
  }
  return useStore(store, selector)
}
