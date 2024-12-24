// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist, StateStorage } from 'zustand/middleware'
import { CreateSessionArgs, Session } from '@roochnetwork/rooch-sdk'

export type SessionActions = {
  addSession: (session: Session) => void
  setCurrentSession: (session?: Session) => void
  removeSession: (session: Session) => void
}

export type SessionStoreState = {
  sessions: Session[]
  currentSession: Session | null
  sessionConf: CreateSessionArgs | undefined
} & SessionActions

export type SessionStore = ReturnType<typeof createSessionStore>

type SessionConfiguration = {
  storage: StateStorage
  storageKey: string
  sessionConf?: CreateSessionArgs
}

export function createSessionStore({ storage, storageKey, sessionConf }: SessionConfiguration) {
  return createStore<SessionStoreState>()(
    persist(
      (set, get) => ({
        sessions: [],
        currentSession: null,
        sessionConf: sessionConf,
        addSession(session) {
          const cache = get().sessions
          cache.push(session)
          set(() => ({
            sessions: cache,
          }))
        },
        setCurrentSession(session) {
          if (!session) {
            set(() => ({
              currentSession: null,
            }))
          } else {
            const cache = get().sessions
            if (!cache.find((item) => item.getAuthKey() === session.getAuthKey())) {
              cache.push(session)
            }
            set(() => ({
              currentSession: session,
              sessions: cache,
            }))
          }
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
