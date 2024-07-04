// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Address, BitcoinAddress, RoochAddress } from '../address/index.js'
import { Authenticator } from '../crypto/index.js'
import { Transaction } from '../transactions/index.js'
import { Bytes } from '../types/index.js'

import type { PublicKey } from './publickey.js'
import type { SignatureScheme } from './signatureScheme.js'

export abstract class Signer {
  abstract sign(input: Bytes): Promise<Bytes>

  abstract signTransaction(input: Transaction): Promise<Authenticator>
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
}
