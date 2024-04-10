// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { StateStorage } from 'zustand/middleware'

export function createInMemoryStore(): StateStorage {
  const store = new Map()
  return {
    getItem(key: string) {
      return store.get(key)
    },
    setItem(key: string, value: string) {
      store.set(key, value)
    },
    removeItem(key: string) {
      store.delete(key)
    },
  }
}

export enum StorageType {
  Session,
  Local,
}

export function getDefaultStorage(type?: StorageType): StateStorage {
  let storage: StateStorage | undefined

  switch (type) {
    case StorageType.Session:
      storage = typeof window !== 'undefined' && window.sessionStorage ? sessionStorage : undefined
      break
    case StorageType.Local:
      storage = typeof window !== 'undefined' && window.localStorage ? localStorage : undefined
  }

  if (!storage) {
    storage = createInMemoryStore()
  }

  return storage
}
