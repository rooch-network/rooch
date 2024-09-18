// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bech32 } from '@scure/base'
import { RoochAddress } from './rooch.js'
import { BitcoinAddress } from './bitcoin.js'
import { Bytes } from '../types/bytes.js'

const PREFIX_BECH32_PUBLIC_KEY = 'npub'

export class NoStrAddress {
  private readonly str: string
  private readonly bytes: Bytes

  constructor(input: Bytes) {
    this.bytes = input
    this.str = bech32.encode(PREFIX_BECH32_PUBLIC_KEY, bech32.toWords(input))
  }

  // TODO: Is that right?
  genRoochAddress(): RoochAddress {
    return BitcoinAddress.fromPublicKey(this.bytes).genRoochAddress()
  }

  toStr(): string {
    return this.str
  }

  toBytes(): Bytes {
    return this.bytes
  }
}
