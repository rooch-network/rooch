// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { TypeArgs } from '../transactions/index.js'

export function normalizeTypeArgs(input: TypeArgs) {
  if ('target' in input) {
    const data = input.target.split('::')

    if (data.length !== 3) {
      throw new Error('invalid type')
    }

    return data
  }

  return [input.address, input.module, input.name]
}

export function normalizeTypeArgsToStr(input: TypeArgs): string {
  if ('target' in input) {
    if (input.target.split('::').length !== 3) {
      throw new Error('invalid type')
    }

    return input.target
  }

  return `${input.address}::${input.module}::${input.name}`
}
