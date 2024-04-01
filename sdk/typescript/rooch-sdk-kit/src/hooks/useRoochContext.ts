// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useContext } from 'react'

import { RoochClientContext } from '../provider'

export function useRoochContext() {
  const ctx = useContext(RoochClientContext)

  if (!ctx) {
    throw new Error(
      'Could not find RoochClientContext. Ensure that you have set up the RoochClientProvider',
    )
  }

  return ctx
}
