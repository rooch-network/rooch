// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'
import { RoochClient } from '@roochnetwork/rooch-sdk'

import { RoochClientContext } from '../provider'

export function useRoochClientContext() {
  const client = useContext(RoochClientContext)

  if (!client) {
    throw new Error(
      'Could not find RoochClientContext. Ensure that you have set up the RoochClientProvider',
    )
  }

  return client
}

export function useRoochClient(): RoochClient {
  return useRoochClientContext().client
}
