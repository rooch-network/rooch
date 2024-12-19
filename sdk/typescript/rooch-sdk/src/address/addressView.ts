// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Address } from './address.js'
import { BitcoinAddress, BitcoinNetowkType } from './bitcoin.js'
import { NoStrAddress } from './nostr.js'
import { RoochAddress } from './rooch.js'
import { Bytes } from '../types/index.js'

export class AddressView extends Address {
  public readonly bitcoinAddress: BitcoinAddress
  public readonly noStrAddress: NoStrAddress
  public readonly roochAddress: RoochAddress

  constructor(publicKey: Bytes, network: BitcoinNetowkType = BitcoinNetowkType.Regtest) {
    const bitcoinAddress = BitcoinAddress.fromPublicKey(publicKey, network)
    const noStrAddress = new NoStrAddress(publicKey)
    const roochAddress = bitcoinAddress.genRoochAddress()
    super(bitcoinAddress.toStr())
    this.bitcoinAddress = bitcoinAddress
    this.noStrAddress = noStrAddress
    this.roochAddress = roochAddress
  }

  toBytes(): Uint8Array {
    return this.roochAddress.toBytes()
  }
  toStr(): string {
    return this.roochAddress.toStr()
  }
}
