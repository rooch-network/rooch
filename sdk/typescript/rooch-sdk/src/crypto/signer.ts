// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { sha3_256 } from '@noble/hashes/sha3'

import { Address, BitcoinAddress, RoochAddress } from '@/address'
import { Authenticator } from '@/crypto'
import { Transaction } from '@/transactions'
import { Bytes } from '@/types'
import { isBytes } from '@/utils'

import type { PublicKey } from './publickey'
import type { SignatureScheme } from './signatureScheme'

export abstract class Signer {
  abstract sign(input: Bytes): Promise<Bytes>

  async signTransaction(input: Bytes | Transaction): Promise<Authenticator> {
    const digest = isBytes(input) ? sha3_256(input as Bytes) : (input as Transaction).hashData()

    return this.signTransactionImp(digest)
  }

  abstract getBitcoinAddress(): BitcoinAddress

  abstract getRoochAddress(): RoochAddress

  /**
   * Get the key scheme of the keypair: Secp256k1 or ED25519
   */
  abstract getKeyScheme(): SignatureScheme

  /**
   * The public key for this keypair
   */
  abstract getPublicKey(): PublicKey<Address>

  protected abstract signTransactionImp(input: Bytes): Promise<Authenticator>
}
