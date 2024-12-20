// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bech32 } from '@scure/base'
import { RoochAddress } from './rooch.js'
import { BitcoinAddress } from './bitcoin.js'
import { Bytes } from '../types/bytes.js'
import { Address } from './address.js'

const PREFIX_BECH32_PUBLIC_KEY = 'npub'

export class NoStrAddress extends Address {
  private readonly bytes: Bytes
  constructor(input: string | Bytes) {
    let raw: string
    let bytes: Bytes
    if (typeof input === 'string') {
      raw = input
      bytes = bech32.decodeToBytes(input).bytes
    } else {
      bytes = input
      raw = bech32.encode(PREFIX_BECH32_PUBLIC_KEY, bech32.toWords(input), false)
    }
    super(raw)
    this.bytes = bytes
  }

  genRoochAddress(): RoochAddress {
    return BitcoinAddress.fromPublicKey(this.bytes).genRoochAddress()
  }

  toBytes(): Bytes {
    return this.bytes
  }
}
