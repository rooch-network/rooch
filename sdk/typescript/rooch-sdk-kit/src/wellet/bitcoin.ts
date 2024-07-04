// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  Address,
  Authenticator,
  BitcoinAddress,
  BitcoinSignMessage,
  PublicKey,
  RoochAddress,
  Secp256k1PublicKey,
  SignatureScheme,
  fromHEX,
  Transaction,
} from '@roochnetwork/rooch-sdk'

import { SupportChain } from '../feature/index.js'
import { Wallet } from '../wellet/index.js'

export abstract class BitcoinWallet extends Wallet {
  async signTransaction(input: Transaction): Promise<Authenticator> {
    const message = new BitcoinSignMessage(input.hashData(), input.getInfo() || '')
    return Authenticator.bitcoin(message, this, 'raw')
  }

  getPublicKey(): PublicKey<Address> {
    if (!this.publicKey) {
      throw Error('Please connect your wallet first')
    }

    return new Secp256k1PublicKey(fromHEX(this.publicKey))
  }

  getRoochAddress(): RoochAddress {
    if (!this.currentAddress) {
      throw Error('Please connect your wallet first')
    }
    return (this.currentAddress as BitcoinAddress).genRoochAddress()
  }

  getBitcoinAddress(): BitcoinAddress {
    if (!this.currentAddress) {
      throw Error('Please connect your wallet first')
    }
    return this.currentAddress as BitcoinAddress
  }

  getKeyScheme(): SignatureScheme {
    return 'Secp256k1'
  }

  normalize_recovery_id(v: number) {
    let normalizeV = v - 27 - 4

    if (normalizeV < 0) {
      normalizeV = normalizeV + 4
    }

    return normalizeV
  }

  switchAccount(): void {
    throw new Error('Method not implemented.')
  }

  getChain(): SupportChain {
    return 'bitcoin'
  }
}
