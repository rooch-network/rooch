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
import { HTTPTransport, requestCallbackType } from '../http/httpTransport.js'
import { useTriggerError, useTriggerRequest } from './globalProvider.js'

const DEFAULT_CREATE_CLIENT = (
  _name: string,
  config: NetworkConfig | RoochClient,
  requestErrorCallback: requestCallbackType,
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
  const triggerError = useTriggerError()
  const triggerRequest = useTriggerRequest()
  const _requestErrorCallback = useCallback<requestCallbackType>(
    (state, error) => {
      try {
        if (state === 'error') {
          if (
            error!.code === ErrorValidateInvalidAccountAuthKey ||
            error!.code === ErrorValidateSessionIsExpired
          ) {
            if (currentSession) {
              removeSession(currentSession)
            }
          }
          triggerError(error!)
        } else {
          triggerRequest(state)
        }
      } catch (e) {
        console.error(e)
      }
    },
    [triggerError, currentSession, removeSession, triggerRequest],
  )

  return useMemo(() => {
    return DEFAULT_CREATE_CLIENT(currentNetwork, networks[currentNetwork], _requestErrorCallback)
  }, [currentNetwork, networks, _requestErrorCallback])
}
