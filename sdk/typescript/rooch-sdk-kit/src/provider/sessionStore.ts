// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist, StateStorage } from 'zustand/middleware'
import { Session } from '@roochnetwork/rooch-sdk'

export type SessionActions = {
  addSession: (session: Session) => void
  setCurrentSession: (session?: Session) => void
  removeSession: (session: Session) => void
}

export type SessionStoreState = {
  sessions: Session[]
  currentSession: Session | null
} & SessionActions

export type SessionStore = ReturnType<typeof createSessionStore>

type ClientConfiguration = {
  storage: StateStorage
  storageKey: string
}

export function createSessionStore({ storage, storageKey }: ClientConfiguration) {
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
            sessions: cacheSessions.filter((c) => c.getAuthKey() !== session.getAuthKey()),
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage, {
          reviver: (key, value) => {
            if (key === 'sessions') {
              return (value as any[]).map((session: any) => Session.fromJson(session))
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
