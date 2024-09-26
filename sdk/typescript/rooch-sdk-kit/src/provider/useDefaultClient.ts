// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useCallback, useMemo } from 'react'
import { isRoochClient, RoochClient } from '@roochnetwork/rooch-sdk'
import { useSessionStore } from '../hooks/useSessionsStore.js'
import { NetworkConfig } from '../hooks/index.js'
import { NetworkConfigs } from './clientProvider.js'
import { HTTPTransport } from '../http/httpTransport.js'

const DEFAULT_CREATE_CLIENT = (
  _name: string,
  config: NetworkConfig | RoochClient,
  setCurrentSession: any,
) => {
  if (isRoochClient(config)) {
    return config
  }

  config.transport = new HTTPTransport(
    {
      url: config.url!.toString(),
    },
    setCurrentSession,
  )

  return new RoochClient(config)
}

interface UseRoochClientParams {
  currentNetwork: string
  networks: NetworkConfigs
}

export function useDefaultClient(params: UseRoochClientParams) {
  const { currentNetwork, networks } = params

  const currentSession = useSessionStore((state) => state.currentSession)
  const removeSession = useSessionStore((state) => state.removeSession)
  const clearSession = useCallback(() => {
    try {
      if (currentSession) {
        removeSession(currentSession)
      }
    } catch (e) {
      console.error(e)
    }
  }, [removeSession, currentSession])

  const client = useMemo(() => {
    return DEFAULT_CREATE_CLIENT(currentNetwork, networks[currentNetwork], clearSession)
  }, [currentNetwork, networks, clearSession])

  return client
}
