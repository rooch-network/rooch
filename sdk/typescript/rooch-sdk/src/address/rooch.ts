// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bech32m } from '@scure/base'

import { Bytes } from '@/types'
import { fromHEX, isHex, toHEX } from '@/utils'

import { Address } from './address'
import { normalizeRoochAddress } from './util'

export class RoochAddress implements Address {
  private readonly address: Bytes

  constructor(address: Bytes | string) {
    if (typeof address === 'string') {
      if (isHex(address)) {
        this.address = fromHEX(address)
      } else {
        this.address = bech32m.fromWords(bech32m.decode(address).words)
      }
    } else {
      this.address = address
    }
  }

  toStr(): string {
    return this.toBech32Address()
  }

  toBytes(): Bytes {
    return this.address
  }

  toHexAddress(): string {
    return normalizeRoochAddress(toHEX(this.address))
  }

  toBech32Address(): string {
    return bech32m.encode('rooch', bech32m.toWords(this.address))
  }
}
