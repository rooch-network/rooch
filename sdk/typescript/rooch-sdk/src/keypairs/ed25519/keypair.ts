// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import nacl from 'tweetnacl'

import { BitcoinAddress, RoochAddress } from '../../address/index.js'
import { Bytes } from '../../types/index.js'
import {
  Authenticator,
  encodeRoochSercetKey,
  isValidHardenedPath,
  Keypair,
  mnemonicToSeedHex,
  PRIVATE_KEY_SIZE,
  decodeRoochSercetKey,
  SignatureScheme,
} from '../../crypto/index.js'

import { derivePath } from './ed25519-hd-key.js'
import { Ed25519PublicKey } from './publickey.js'
import { Transaction } from '../../transactions/index.js'

export const DEFAULT_ED25519_DERIVATION_PATH = `m/44'/784'/0'/0'/0'`

/**
 * Ed25519 Keypair data. The publickey is the 32-byte public key and
 * the secretkey is 64-byte, where the first 32 bytes is the secret
 * key and the last 32 bytes is the public key.
 */
export interface Ed25519KeypairData {
  publicKey: Bytes
  secretKey: Bytes
}

/**
 * An Ed25519 Keypair used for signing transactions.
 */
export class Ed25519Keypair extends Keypair {
  private keypair: Ed25519KeypairData

  /**
   * Create a new Ed25519 keypair instance.
   * Generate random keypair if no {@link Ed25519Keypair} is provided.
   *
   * @param keypair Ed25519 keypair
   */
  constructor(keypair?: Ed25519KeypairData) {
    super()
    if (keypair) {
      this.keypair = keypair
    } else {
      this.keypair = nacl.sign.keyPair()
    }
  }

  /**
   * Get the key scheme of the keypair ED25519
   */
  getKeyScheme(): SignatureScheme {
    return 'ED25519'
  }

  /**
   * Generate a new random Ed25519 keypair
   */
  static generate(): Ed25519Keypair {
    return new Ed25519Keypair(nacl.sign.keyPair())
  }

  /**
   * Create a Ed25519 keypair from a raw secret key byte array, also known as seed.
   * This is NOT the private scalar which is result of hashing and bit clamping of
   * the raw secret key.
   *
   * @throws error if the provided secret key is invalid and validation is not skipped.
   *
   * @param secretKey secret key byte array
   * @param skipValidation skip secret key validation
   */
  static fromSecretKey(secretKey: Uint8Array | string, skipValidation?: boolean): Ed25519Keypair {
    const decodeSecretKey =
      typeof secretKey === 'string'
        ? (() => {
            const decoded = decodeRoochSercetKey(secretKey)
            if (decoded.schema !== 'ED25519') {
              throw new Error('provided secretKey is invalid')
            }
            return decoded.secretKey
          })()
        : secretKey

    const secretKeyLength = decodeSecretKey.length
    if (secretKeyLength !== PRIVATE_KEY_SIZE) {
      throw new Error(
        `Wrong secretKey size. Expected ${PRIVATE_KEY_SIZE} bytes, got ${secretKeyLength}.`,
      )
    }
    const keypair = nacl.sign.keyPair.fromSeed(decodeSecretKey)
    if (!skipValidation) {
      const encoder = new TextEncoder()
      const signData = encoder.encode('rooch validation')
      const signature = nacl.sign.detached(signData, keypair.secretKey)
      if (!nacl.sign.detached.verify(signData, signature, keypair.publicKey)) {
        throw new Error('provided secretKey is invalid')
      }
    }
    return new Ed25519Keypair(keypair)
  }

  getBitcoinAddress(): BitcoinAddress {
    throw new Error('Method not implemented in Ed25519.')
  }

  getRoochAddress(): RoochAddress {
    return this.getPublicKey().toAddress()
  }

  /**
   * The public key for this Ed25519 keypair
   */
  getPublicKey(): Ed25519PublicKey {
    return new Ed25519PublicKey(this.keypair.publicKey)
  }

  /**
   * The Bech32 secret key string for this Ed25519 keypair
   */
  getSecretKey(): string {
    return encodeRoochSercetKey(
      this.keypair.secretKey.slice(0, PRIVATE_KEY_SIZE),
      this.getKeyScheme(),
    )
  }

  /**
   * Return the signature for the provided data using Ed25519.
   */
  async sign(input: Bytes) {
    return nacl.sign.detached(input, this.keypair.secretKey)
  }

  signTransaction(input: Transaction): Promise<Authenticator> {
    return Authenticator.rooch(input.hashData(), this)
  }

  /**
   * Derive Ed25519 keypair from mnemonics and path. The mnemonics must be normalized
   * and validated against the english wordlist.
   *
   * If path is none, it will default to m/44'/784'/0'/0'/0', otherwise the path must
   * be compliant to SLIP-0010 in form m/44'/784'/{account_index}'/{change_index}'/{address_index}'.
   */
  static deriveKeypair(mnemonics: string, path?: string): Ed25519Keypair {
    const newPath = path ?? DEFAULT_ED25519_DERIVATION_PATH

    if (!isValidHardenedPath(newPath)) {
      throw new Error('Invalid derivation path')
    }
    const { key } = derivePath(newPath, mnemonicToSeedHex(mnemonics))

    return Ed25519Keypair.fromSecretKey(key)
  }

  /**
   * Derive Ed25519 keypair from mnemonicSeed and path.
   *
   * If path is none, it will default to m/44'/784'/0'/0'/0', otherwise the path must
   * be compliant to SLIP-0010 in form m/44'/784'/{account_index}'/{change_index}'/{address_index}'.
   */
  static deriveKeypairFromSeed(seedHex: string, path?: string): Ed25519Keypair {
    const newPath = path ?? DEFAULT_ED25519_DERIVATION_PATH

    if (!isValidHardenedPath(newPath)) {
      throw new Error('Invalid derivation path')
    }
    const { key } = derivePath(newPath, seedHex)

    return Ed25519Keypair.fromSecretKey(key)
  }
}
