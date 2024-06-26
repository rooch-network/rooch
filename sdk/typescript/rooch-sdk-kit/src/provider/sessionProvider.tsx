// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ReactNode } from 'react'
import { createContext, useRef } from 'react'

import type { StateStorage } from 'zustand/middleware'

import { createSessionStore, SessionStore } from './sessionStore.js'

import { useCurrentNetwork } from '../hooks/index.js'
import { getDefaultStorage, StorageType } from '../utils/index.js'

const DEFAULT_SESSION_STORAGE_KEY = function (key: string | undefined, network: string) {
  if (key) {
    return key
  }

  return 'rooch-sdk-kit:rooch-session-info' + network
}

export const RoochSessionContext = createContext<SessionStore | null>(null)

export type RoochSessionProviderProps = {
  /** Configures how the most recently connected to wallet account is stored. Defaults to using sessionStorage. */
  storage?: StateStorage

  /** The key to use to store the most recently connected wallet account. */
  storageKey?: string

  children: ReactNode
}

export function RoochSessionProvider(props: RoochSessionProviderProps) {
  // ** Props **
  const { storage, storageKey, children } = props

  // ** Hooks **
  const network = useCurrentNetwork()

  const storeRef = useRef(
    createSessionStore({
      storage: storage || getDefaultStorage(StorageType.Session),
      storageKey: DEFAULT_SESSION_STORAGE_KEY(storageKey, network),
    }),
  )
  return (
    <RoochSessionContext.Provider value={storeRef.current}>{children}</RoochSessionContext.Provider>
  )
}
