// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// import { useCallback, useMemo } from 'react'
// import {
//   ErrorValidateInvalidAccountAuthKey,
//   ErrorValidateSessionIsExpired,
//   isRoochClient,
//   RoochClient,
//   RoochWebSocketTransport,
// } from '@roochnetwork/rooch-sdk'
// import { useSessionStore } from '../hooks/useSessionsStore.js'
// import { NetworkConfig } from '../hooks/index.js'
// import { NetworkConfigs } from './clientProvider.js'
// import { requestCallbackType } from '../http/httpTransport.js'
// import { useTriggerError, useTriggerRequest } from './globalProvider.js'

// const DEFAULT_CREATE_WS_CLIENT = (
//   _name: string,
//   config: NetworkConfig | RoochClient,
//   _: requestCallbackType,
// ) => {
//   if (isRoochClient(config)) {
//     return config
//   }

//   const wsTransport = new RoochWebSocketTransport({
//     url: config.url!.toString(),
//     requestTimeout: 5000,
//     maxReconnectAttempts: 3,
//   })

//   config.transport = wsTransport
//   config.subscriptionTransport = wsTransport

//   return new RoochClient(config)
// }

// interface UseRoochWSClientParams {
//   currentNetwork: string
//   networks: NetworkConfigs
// }

// export function useDefaultWSClient(params: UseRoochWSClientParams) {
//   const { currentNetwork, networks } = params

//   const currentSession = useSessionStore((state) => state.currentSession)
//   const removeSession = useSessionStore((state) => state.removeSession)
//   const triggerError = useTriggerError()
//   const triggerRequest = useTriggerRequest()
//   const _requestErrorCallback = useCallback<requestCallbackType>(
//     (state, error) => {
//       try {
//         if (state === 'error') {
//           if (
//             error!.code === ErrorValidateInvalidAccountAuthKey ||
//             error!.code === ErrorValidateSessionIsExpired
//           ) {
//             if (currentSession) {
//               removeSession(currentSession)
//             }
//           }
//           triggerError(error!)
//         }
//         triggerRequest(state)
//       } catch (e) {
//         console.error(e)
//       }
//     },
//     [triggerError, currentSession, removeSession, triggerRequest],
//   )

//   return useMemo(() => {
//     return DEFAULT_CREATE_WS_CLIENT(currentNetwork, networks[currentNetwork], _requestErrorCallback)
//   }, [currentNetwork, networks, _requestErrorCallback])
// }
