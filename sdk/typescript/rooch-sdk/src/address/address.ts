// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'
import { toShortStr } from '../utils/index.js'

export const ROOCH_BECH32_PREFIX = 'rooch'

export const ROOCH_ADDRESS_LENGTH = 32

export abstract class Address {
  protected rawAddress: string

  constructor(input: string) {
    this.rawAddress = input
  }

  abstract toBytes(): Bytes

  toStr(): string {
    return this.rawAddress
  }

  toShortStr(
    shortOpt: { start: number; end: number } = {
      start: 6,
      end: 6,
    },
  ): string {
    return toShortStr(this.rawAddress, shortOpt)
  }
}
