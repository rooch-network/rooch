// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode, useEffect, useState } from 'react'
import { createContext, useRef } from 'react'

import { SessionStore } from './sessionStore.js'
import { getDefaultStorage, StorageType } from '../utils/index.js'
import { useSession } from '../hooks'
import { createTelegramStore } from './telegramStore'
import { useRoochClient } from '@roochnetwork/rooch-sdk-kit'

const DEFAULT_SESSION_STORAGE_KEY = function (_?: string) {
  return 'rooch-sdk-kit:rooch-session-info'
}

export const TelegramContext = createContext<SessionStore | null>(null)

export type TelegramProviderProps = {
  children: ReactNode
  waitEnvCheck: ReactNode
}

export function TelegramProvider(props: TelegramProviderProps) {
  return (
    <TelegramContext.Provider value={storeRef.current}>
      <TelegramManager children={children} waitEnvCheck={waitEnvCheck} />
    </TelegramContext.Provider>
  )
}

type WalletConnectionManagerProps = Required<
  Pick<TelegramProviderProps, 'children' | 'waitEnvCheck'>
>

function TelegramManager({ children, waitEnvCheck }: WalletConnectionManagerProps) {
  const [checking, setChecking] = useState(true)
  const curSession = useSession()
  const client = useRoochClient()

  useEffect(() => {
    if (curSession) {
      setChecking(false)
      return
    }

    client.executeViewFunction('0x3::')
  }, [curSession])

  return checking ? waitEnvCheck : children
}
