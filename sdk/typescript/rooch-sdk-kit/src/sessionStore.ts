// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist, StateStorage } from 'zustand/middleware'
import { RoochSessionAccount } from '@roochnetwork/rooch-sdk'

export type SessionActions = {
  addSession: (session: RoochSessionAccount) => void
  setCurrentSession: (session: RoochSessionAccount) => void
  removeSession: (session: RoochSessionAccount) => void
}

export type SessionStore = {
  sessions: RoochSessionAccount[]
  currentSession: RoochSessionAccount | null
} & SessionActions

type ClientConfiguration = {
  storage: StateStorage
  storageKey: string
}

export function createSessionStore({ storage, storageKey }: ClientConfiguration) {
  return createStore<SessionStore>()(
    persist(
      (set, get) => ({
        sessions: [],
        currentSession: null,
        addSession(session) {
          const cache = get().sessions
          set(() => ({
            sessions: cache.concat(session),
          }))
        },
        setCurrentSession(session) {
          set(() => ({
            currentSession: session,
          }))
        },
        removeSession(session) {
          const cache = get().sessions
          set(() => ({
            sessions: cache.filter(
              async (c) => (await c.getRoochAddress()) === (await session.getRoochAddress()),
            ),
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage),
        partialize: ({ sessions, currentSession }) => ({
          sessions,
          currentSession,
        }),
      },
    ),
  )
}
