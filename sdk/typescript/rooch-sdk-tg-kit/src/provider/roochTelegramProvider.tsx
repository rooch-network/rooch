// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode, useEffect, useState } from 'react'
import { createContext, useRef } from 'react'

import { TelegramStore } from './telegramStore.js'
import { getDefaultStorage, StorageType } from '../utils/index.js'
import { useSession } from '../hooks'
import { createTelegramStore } from './telegramStore'
import { useRoochClient } from '@roochnetwork/rooch-sdk-kit'

const DEFAULT_SESSION_STORAGE_KEY = function (_?: string) {
  return 'rooch-sdk-kit:rooch-session-info'
}

export const TelegramContext = createContext<TelegramStore | null>(null)

export type TelegramProviderProps = {
  children: ReactNode
  waitEnvCheck: ReactNode
}

export function TelegramProvider(props: TelegramProviderProps) {
  const { children, waitEnvCheck } = props
  const storeRef = useRef(
      createTelegramStore({
        storage: getDefaultStorage(StorageType.Local),
        storageKey: DEFAULT_SESSION_STORAGE_KEY(),
      }),
  )
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
