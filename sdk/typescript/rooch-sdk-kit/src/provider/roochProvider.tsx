// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { ReactNode } from 'react'
import { createContext, useRef } from 'react'

import { NetworkConfigs, RoochClientProvider } from './clientProvider.js'
import { createSessionStore, SessionStore } from './roochStore.js'
import { getDefaultStorage, StorageType } from '../utils/index.js'

const DEFAULT_SESSION_STORAGE_KEY = function (_?: string) {
  return 'rooch-sdk-kit:rooch-session-info'
}

export const RoochContext = createContext<SessionStore | null>(null)

export type RoochProviderProps<T extends NetworkConfigs> = {
  networks?: NetworkConfigs
  onNetworkChange?: (network: keyof T & string) => void
  children: ReactNode
} & (
  | {
      defaultNetwork?: keyof T & string
      network?: never
    }
  | {
      defaultNetwork?: never
      network?: keyof T & string
    }
)

export function RoochProvider<T extends NetworkConfigs>(props: RoochProviderProps<T>) {
  // ** Props **
  const { children, networks, defaultNetwork } = props

  const storeRef = useRef(
    createSessionStore({
      storage: getDefaultStorage(StorageType.Local),
      storageKey: DEFAULT_SESSION_STORAGE_KEY(),
    }),
  )
  return (
    <RoochContext.Provider value={storeRef.current}>
      <RoochClientProvider networks={networks} defaultNetwork={defaultNetwork}>
        {children}
      </RoochClientProvider>
    </RoochContext.Provider>
  )
}
