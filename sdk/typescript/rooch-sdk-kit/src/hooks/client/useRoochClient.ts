// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'

import { ClientManagerContext } from '../../provider'

export function useRoochClient() {
  const client = useContext(ClientManagerContext)

  if (!client) {
    throw new Error(
      'Could not find RoochClient. Ensure that you have set up the RoochClientProvider.',
    )
  }
  return client
}
