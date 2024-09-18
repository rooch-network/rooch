// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'
import { BitcoinAddress, BitcoinNetowkType } from './bitcoin.js'
import { NoStrAddress } from './nostr.js'
import { RoochAddress } from './rooch.js'

export const ROOCH_BECH32_PREFIX = 'rooch'

export const ROOCH_ADDRESS_LENGTH = 32

export interface Address {
  toBytes(): Bytes
  toStr(): string
}

export class AddressView implements Address {
  public readonly bitcoinAddress: BitcoinAddress
  public readonly noStrAddress: NoStrAddress
  public readonly roochAddress: RoochAddress

  constructor(publicKey: Bytes, network: BitcoinNetowkType = BitcoinNetowkType.Regtest) {
    this.bitcoinAddress = BitcoinAddress.fromPublicKey(publicKey, network)
    this.noStrAddress = new NoStrAddress(publicKey)
    this.roochAddress = this.bitcoinAddress.genRoochAddress()
  }

  toBytes(): Uint8Array {
    return this.roochAddress.toBytes()
  }
  toStr(): string {
    return this.roochAddress.toStr()
  }
}
