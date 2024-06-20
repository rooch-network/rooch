// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'

import { RoochClientContext, RoochClientProviderContext } from '@/provider/clientProvider'

export function useRoochContext(): RoochClientProviderContext {
  const context = useContext(RoochClientContext)
  if (!context) {
    throw new Error(
      'Could not find RoochClientContext. Ensure that you have set up the RoochClientProvider.',
    )
  }
  return context
}
