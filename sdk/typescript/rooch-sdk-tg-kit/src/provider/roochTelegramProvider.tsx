// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {ReactNode, useEffect, useState} from 'react'
import { createContext, useRef } from 'react'

import { NetworkConfigs, RoochClientProvider } from './clientProvider.js'
import { createSessionStore, SessionStore } from './sessionStore.js'
import { getDefaultStorage, StorageType } from '../utils/index.js'

const DEFAULT_SESSION_STORAGE_KEY = function (_?: string) {
  return 'rooch-sdk-kit:rooch-session-info'
}

export const RoochContext = createContext<SessionStore | null>(null)

export type RoochTelegramProvider = {
    children: ReactNode,
    waitEnvCheck?: ReactNode,
}

export function RoochProvider(props: RoochTelegramProvider) {
  // ** Props **
  const { children, waitEnvCheck } = props
  const [loading, setLoading] = useState(true)

    useEffect(() => {
        
    })

  const storeRef = useRef(
    createSessionStore({
      storage: getDefaultStorage(StorageType.Local),
      storageKey: DEFAULT_SESSION_STORAGE_KEY(),
    }),
  )
  return (
      loading? waitEnvCheck: <>{children}</>
  )
}
