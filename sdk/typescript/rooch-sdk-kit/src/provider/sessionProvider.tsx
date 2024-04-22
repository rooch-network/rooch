// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createContext, useEffect, useRef, useState } from 'react'
import type { ReactNode } from 'react'

import type { StateStorage } from 'zustand/middleware'
import { createSessionStore, SessionStore } from '../sessionStore'
import { useRoochClient } from '../hooks'
import { useCurrentNetwork } from '../hooks'
import { Network } from '@roochnetwork/rooch-sdk'
import { getDefaultStorage, StorageType } from '../utils/stateStorage'

const DEFAULT_SESSION_STORAGE_KEY = function (key: string | undefined, network: Network) {
  if (key) {
    return key
  }

  return 'rooch-sdk-kit:rooch-session-info' + network.id
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

  // ** State **
  const [init, setInit] = useState(true)

  // ** Hooks **
  const client = useRoochClient()
  const network = useCurrentNetwork()
  const sessionStoreRef = useRef<SessionStore>()

  // init
  useEffect(() => {
    const init = async () => {
      sessionStoreRef.current = createSessionStore({
        client: client,
        storage: storage || getDefaultStorage(StorageType.Session),
        storageKey: DEFAULT_SESSION_STORAGE_KEY(storageKey, network),
      })
    }

    init().finally(() => setInit(false))
  }, [client, network, storage, storageKey])

  return init ? (
    <></>
  ) : (
    <RoochSessionContext.Provider value={sessionStoreRef.current!}>
      {children}
    </RoochSessionContext.Provider>
  )
}
