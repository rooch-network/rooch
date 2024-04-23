// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist, StateStorage } from 'zustand/middleware'
import { RoochClient, RoochSessionAccount } from '@roochnetwork/rooch-sdk'
import { WalletRoochSessionAccount } from './types'

export type SessionActions = {
  addSession: (session: RoochSessionAccount) => void
  setCurrentSession: (session?: RoochSessionAccount) => void
  removeSession: (session: RoochSessionAccount) => void
}

export type SessionStoreState = {
  sessions: RoochSessionAccount[]
  currentSession: RoochSessionAccount | null
} & SessionActions

export type SessionStore = ReturnType<typeof createSessionStore>

type ClientConfiguration = {
  client: RoochClient
  storage: StateStorage
  storageKey: string
  // session: RoochSessionAccount[]
  // currentSession?: RoochSessionAccount
}

export function createSessionStore({ client, storage, storageKey }: ClientConfiguration) {
  return createStore<SessionStoreState>()(
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
          if (!session) {
            set(() => ({
              currentSession: null,
            }))
            return
          }
          const cache = get().sessions
          if (!cache.find((item) => item.getAuthKey() === session.getAuthKey())) {
            cache.push(session)
          }
          set(() => ({
            currentSession: session,
            sessions: cache,
          }))
        },
        removeSession(session) {
          const cacheSessions = get().sessions
          const cacheCurSession = get().currentSession
          set(() => ({
            currentSession:
              cacheCurSession?.getAuthKey() === session.getAuthKey() ? null : cacheCurSession,
            sessions: cacheSessions.filter((c) => c.getAddress() !== session.getAddress()),
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage, {
          reviver: (key, value) => {
            if (key === 'sessions') {
              return (value as any[]).map((session: any) =>
                WalletRoochSessionAccount.formJson(session, client),
              )
            }

            return value
          },
        }),
        partialize: ({ sessions }) => ({
          sessions,
        }),
      },
    ),
  )
}
