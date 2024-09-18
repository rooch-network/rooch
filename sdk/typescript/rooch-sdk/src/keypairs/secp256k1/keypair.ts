// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { HDKey } from '@scure/bip32'

import { schnorr, secp256k1 } from '@noble/curves/secp256k1'
import { BitcoinAddress, RoochAddress } from '../../address/index.js'
import {
  Authenticator,
  BitcoinSignMessage,
  encodeRoochSercetKey,
  isValidBIP32Path,
  Keypair,
  mnemonicToSeed,
  decodeRoochSercetKey,
  SignatureScheme,
} from '../../crypto/index.js'
import { Bytes } from '../../types/index.js'
import { blake2b, sha256, toHEX } from '../../utils/index.js'
import { Secp256k1PublicKey } from './publickey.js'
import { Transaction } from '../../transactions/index.js'

export const DEFAULT_SECP256K1_DERIVATION_PATH = "m/54'/784'/0'/0/0"

/**
 * Secp256k1 Keypair data
 */
export interface Secp256k1KeypairData {
  publicKey: Bytes
  secretKey: Bytes
}

/**
 * An Secp256k1 Keypair used for signing transactions.
 */
export class Secp256k1Keypair extends Keypair {
  private keypair: Secp256k1KeypairData

  /**
   * Create a new keypair instance.
   * Generate random keypair if no {@link Secp256k1Keypair} is provided.
   *
   * @param keypair secp256k1 keypair
   */
  constructor(keypair?: Secp256k1KeypairData) {
    super()
    if (keypair) {
      this.keypair = keypair
    } else {
      const secretKey: Uint8Array = secp256k1.utils.randomPrivateKey()
      const publicKey: Uint8Array = secp256k1.getPublicKey(secretKey, true)

      this.keypair = { publicKey, secretKey }
    }
  }

  getBitcoinAddress(): BitcoinAddress {
    return this.getSchnorrPublicKey().toAddress().bitcoinAddress
  }

  getRoochAddress(): RoochAddress {
    return this.getSchnorrPublicKey().toAddress().roochAddress
  }

  /**
   * Get the key scheme of the keypair Secp256k1
   */
  getKeyScheme(): SignatureScheme {
    return 'Secp256k1'
  }

  /**
   * Generate a new random keypair
   */
  static generate(): Secp256k1Keypair {
    return new Secp256k1Keypair()
  }

  /**
   * Create a keypair from a raw secret key byte array.
   *
   * This method should only be used to recreate a keypair from a previously
   * generated secret key. Generating keypairs from a random seed should be done
   * with the {@link Keypair.fromSeed} method.
   *
   * @throws error if the provided secret key is invalid and validation is not skipped.
   *
   * @param secretKey secret key byte array
   * @param skipValidation skip secret key validation
   */

  static fromSecretKey(secretKey: Uint8Array | string, skipValidation?: boolean): Secp256k1Keypair {
    const decodeSecretKey =
      typeof secretKey === 'string'
        ? (() => {
            const decoded = decodeRoochSercetKey(secretKey)
            if (decoded.schema !== 'Secp256k1') {
              throw new Error('provided secretKey is invalid')
            }
            return decoded.secretKey
          })()
        : secretKey

    const publicKey: Uint8Array = secp256k1.getPublicKey(decodeSecretKey, true)
    if (!skipValidation) {
      const encoder = new TextEncoder()
      const signData = encoder.encode('rooch validation')
      const msgHash = toHEX(blake2b(signData, { dkLen: 32 }))
      const signature = secp256k1.sign(msgHash, decodeSecretKey)
      if (!secp256k1.verify(signature, msgHash, publicKey, { lowS: true })) {
        throw new Error('Provided secretKey is invalid')
      }
    }
    return new Secp256k1Keypair({ publicKey, secretKey: decodeSecretKey })
  }

  /**
   * Generate a keypair from a 32 byte seed.
   *
   * @param seed seed byte array
   */
  static fromSeed(seed: Uint8Array): Secp256k1Keypair {
    let publicKey = secp256k1.getPublicKey(seed, true)
    return new Secp256k1Keypair({ publicKey, secretKey: seed })
  }

  /**
   * The public key for this keypair
   */
  getPublicKey(): Secp256k1PublicKey {
    return new Secp256k1PublicKey(this.keypair.publicKey)
  }

  getSchnorrPublicKey(): Secp256k1PublicKey {
    return new Secp256k1PublicKey(schnorr.getPublicKey(this.keypair.secretKey))
  }

  /**
   * The Bech32 secret key string for this Secp256k1 keypair
   */
  getSecretKey(): string {
    return encodeRoochSercetKey(this.keypair.secretKey, this.getKeyScheme())
  }

  /**
   * Return the signature for the provided data.
   */
  async sign(input: Bytes) {
    const msgHash = sha256(input)
    const sig = secp256k1.sign(msgHash, this.keypair.secretKey, {
      lowS: true,
    })

    return sig.toCompactRawBytes()
  }

  async signTransaction(input: Transaction): Promise<Authenticator> {
    return await Authenticator.bitcoin(
      new BitcoinSignMessage(input.hashData(), input.getInfo() ?? 'sdk'),
      this,
    )
  }

  /**
   * Derive Secp256k1 keypair from mnemonics and path. The mnemonics must be normalized
   * and validated against the english wordlist.
   *
   * If path is none, it will default to m/54'/784'/0'/0/0, otherwise the path must
   * be compliant to BIP-32 in form m/54'/784'/{account_index}'/{change_index}/{address_index}.
   */
  static deriveKeypair(mnemonics: string, path?: string): Secp256k1Keypair {
    if (path == null) {
      path = DEFAULT_SECP256K1_DERIVATION_PATH
    }
    if (!isValidBIP32Path(path)) {
      throw new Error('Invalid derivation path')
    }
    const key = HDKey.fromMasterSeed(mnemonicToSeed(mnemonics)).derive(path)
    if (key.publicKey == null || key.privateKey == null) {
      throw new Error('Invalid key')
    }
    return new Secp256k1Keypair({
      publicKey: key.publicKey,
      secretKey: key.privateKey,
    })
  }
}
