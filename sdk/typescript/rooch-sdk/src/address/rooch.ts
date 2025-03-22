// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bech32m } from '@scure/base'

import { Bytes } from '../types/index.js'
import { fromHEX, isHex, toHEX } from '../utils/index.js'

import { normalizeRoochAddress } from './util.js'
import { Address, ROOCH_BECH32_PREFIX } from './address.js'

export class RoochAddress extends Address {
  private readonly bytes: Bytes

  constructor(input: Bytes | string) {
    let bytes: Bytes
    if (typeof input === 'string') {
      if (isHex(input)) {
        const normalizeAddress = normalizeRoochAddress(input)
        bytes = fromHEX(normalizeAddress)
      } else {
        bytes = bech32m.decodeToBytes(input).bytes
      }
    } else {
      bytes = input
    }
    super(bech32m.encode(ROOCH_BECH32_PREFIX, bech32m.toWords(bytes), false))
    this.bytes = bytes
  }

  toBytes(): Bytes {
    return this.bytes
  }

  toHexAddress(): string {
    return normalizeRoochAddress(toHEX(this.bytes))
  }

  toBech32Address(): string {
    return this.rawAddress
  }
}
