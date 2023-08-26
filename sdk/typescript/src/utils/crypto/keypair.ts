// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { sha256 } from '@noble/hashes/sha256'
import type { PublicKey } from './publickey'
import { SignatureScheme, toSerializedSignature } from './signature'

export const PRIVATE_KEY_SIZE = 32
export const LEGACY_PRIVATE_KEY_SIZE = 64

export type ExportedKeypair = {
  schema: SignatureScheme
  privateKey: string
}

// interface SignedMessage {
// 	bytes: Uint8Array
// 	signature: SerializedSignature
// }

export abstract class BaseSigner {
  abstract sign(bytes: Uint8Array): Promise<Uint8Array>

  async signMessage(bytes: Uint8Array) {
    const digest = sha256(bytes)

    const signature = toSerializedSignature({
      signature: await this.sign(digest),
      signatureScheme: this.getKeyScheme(),
      pubKey: this.getPublicKey(),
    })

    return {
      signature,
      bytes,
    }
  }

  toRoochAddress(): string {
    return this.getPublicKey().toRoochAddress()
  }

  /**
   * Return the signature for the data.
   * Prefer the async verion {@link sign}, as this method will be deprecated in a future release.
   */
  abstract signData(data: Uint8Array): Uint8Array

  /**
   * Get the key scheme of the keypair: Secp256k1 or ED25519
   */
  abstract getKeyScheme(): SignatureScheme

  /**
   * The public key for this keypair
   */
  abstract getPublicKey(): PublicKey
}

/**
 * TODO: Document
 */
export abstract class Keypair extends BaseSigner {
  abstract export(): ExportedKeypair
}
