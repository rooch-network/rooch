// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Wallet } from '@/wellet/wallet'
import {
  Address,
  Authenticator,
  BitcoinAddress,
  BitcoinSignMessage,
  Bytes,
  PublicKey,
  RoochAddress,
  Secp256k1PublicKey,
  SignatureScheme,
} from '@roochnetwork/rooch-sdk'
import { SupportChain } from '@/feature'

export abstract class BitcoinWallet extends Wallet {
  protected async signTransactionImp(input: Bytes): Promise<Authenticator> {
    return await Authenticator.bitcoin(new BitcoinSignMessage(input, 'sdk'), this)
  }

  getPublicKey(): PublicKey<Address> {
    if (!this.publicKey) {
      throw Error('Please connect your wallet first')
    }
    return new Secp256k1PublicKey(this.publicKey)
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
    return SupportChain.BITCOIN
  }
}
