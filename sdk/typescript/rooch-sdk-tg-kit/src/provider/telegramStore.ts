// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { StateStorage } from 'zustand/middleware'

export type telegramActions = {
  setRoochAddress: (address: string) => void
}

export type TelegramStore = ReturnType<typeof createTelegramStore>

export type TelegramStoreState = {
  roochAddress: string
} & telegramActions

type TelegramConfiguration = {
  storage: StateStorage
  storageKey: string
}

export function createTelegramStore({ storage, storageKey }: TelegramConfiguration) {
  return createStore<TelegramStoreState>()(
    persist(
      (set) => ({
        roochAddress: '',
        setRoochAddress(address: string) {
          set(() => ({
            roochAddress: address,
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage),
        partialize: ({ roochAddress }) => ({
          roochAddress,
        }),
      },
    ),
  )
}
