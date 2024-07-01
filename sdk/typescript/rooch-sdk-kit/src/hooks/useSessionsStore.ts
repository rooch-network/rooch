// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'
import { useStore } from 'zustand'

import { SessionStoreState } from '../provider/roochStore.js'
import { RoochContext } from '../provider/index.js'

export function useRoochSessionStore<T>(selector: (state: SessionStoreState) => T): T {
  const store = useContext(RoochContext)
  if (!store) {
    throw new Error(
      'Could not find RoochSessionContext. Ensure that you have set up the RoochClientProvider.',
    )
  }
  return useStore(store, selector)
}
