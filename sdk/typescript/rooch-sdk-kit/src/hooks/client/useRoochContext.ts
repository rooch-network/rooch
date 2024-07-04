// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'

import { ClientContext, ClientProviderContext } from '../../provider/clientProvider.js'

export function useRoochContext(): ClientProviderContext {
  const context = useContext(ClientContext)
  if (!context) {
    throw new Error(
      'Could not find RoochClientContext. Ensure that you have set up the RoochClientProvider.',
    )
  }
  return context
}
