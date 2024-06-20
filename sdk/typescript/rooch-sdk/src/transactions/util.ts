// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ModuleArgs } from '@/transactions/transactionData'

export function normalizeModuleArgs(input: ModuleArgs) {
  if ('target' in input) {
    const data = input.target.split('::')

    if (data.length !== 3) {
      throw new Error('invalid type')
    }

    return data
  }

  return [input.address, input.module, input.function]
}
