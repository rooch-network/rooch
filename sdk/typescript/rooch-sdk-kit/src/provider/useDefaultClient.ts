// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useCallback, useMemo } from 'react'
import {
  ErrorValidateInvalidAccountAuthKey,
  ErrorValidateSessionIsExpired,
  isRoochClient,
  RoochClient,
} from '@roochnetwork/rooch-sdk'
import { useSessionStore } from '../hooks/useSessionsStore.js'
import { NetworkConfig } from '../hooks/index.js'
import { NetworkConfigs } from './clientProvider.js'
import { HTTPTransport } from '../http/httpTransport.js'
import { useSetError } from './errorProvider.js'

const DEFAULT_CREATE_CLIENT = (
  _name: string,
  config: NetworkConfig | RoochClient,
  requestErrorCallback: (code: number, msg: string) => void,
) => {
  if (isRoochClient(config)) {
    return config
  }

  config.transport = new HTTPTransport(
    {
      url: config.url!.toString(),
    },
    requestErrorCallback,
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
  const setError = useSetError()
  const _requestErrorCallback = useCallback(
    (code: number, msg: string) => {
      try {
        if (code === ErrorValidateInvalidAccountAuthKey || code === ErrorValidateSessionIsExpired) {
          if (currentSession) {
            removeSession(currentSession)
          }
        }
      } catch (e) {
        console.error(e)
      }
      setError({
        code: code,
        msg: msg,
      })
    },
    [removeSession, currentSession, setError],
  )

  return useMemo(() => {
    return DEFAULT_CREATE_CLIENT(currentNetwork, networks[currentNetwork], _requestErrorCallback)
  }, [currentNetwork, networks, _requestErrorCallback])
}
