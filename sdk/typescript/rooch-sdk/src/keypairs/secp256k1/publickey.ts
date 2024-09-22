// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { secp256k1 } from '@noble/curves/secp256k1'

import { AddressView } from '../../address/index.js'
import { PublicKey, PublicKeyInitData, SIGNATURE_SCHEME_TO_FLAG } from '../../crypto/index.js'
import { fromB64, sha256, toHEX } from '../../utils/index.js'

const SCHNORR_PUBLIC_KEY_SIZE = 32

/**
 * A Secp256k1 public key
 */
export class Secp256k1PublicKey extends PublicKey<AddressView> {
  static SIZE = SCHNORR_PUBLIC_KEY_SIZE

  private readonly data: Uint8Array

  /**
   * Create a new Secp256k1PublicKey object
   * @param value secp256k1 public key as buffer or base-64 encoded string
   */
  constructor(value: PublicKeyInitData) {
    super()

    if (typeof value === 'string') {
      this.data = fromB64(value)
    } else if (value instanceof Uint8Array) {
      this.data = value
    } else {
      this.data = Uint8Array.from(value)
    }

    if (this.data.length !== SCHNORR_PUBLIC_KEY_SIZE && this.data.length !== 33) {
      throw new Error(
        `Invalid public key input. Expected ${SCHNORR_PUBLIC_KEY_SIZE} bytes, got ${this.data.length}`,
      )
    }
  }

  /**
   * Checks if two Secp256k1 public keys are equal
   */
  // override equals(publicKey: Secp256k1PublicKey): boolean {
  //   return super.equals(publicKey)
  // }

  equals(publicKey: Secp256k1PublicKey): boolean {
    return super.equals(publicKey)
  }

  /**
   * Return the byte array representation of the Secp256k1 public key
   */
  override toBytes(): Uint8Array {
    return this.data
  }

  override toString(): string {
    return toHEX(this.data)
  }

  /**
   * Return the Bitcoin address associated with this Secp256k1 public key
   */
  override toAddress(): AddressView {
    return new AddressView(this.data)
  }

  /**
   * Return the Rooch address associated with this Secp256k1 public key
   */
  flag(): number {
    return SIGNATURE_SCHEME_TO_FLAG['Secp256k1']
  }

  /**
   * Verifies that the signature is valid for the provided message
   */
  async verify(message: Uint8Array, signature: Uint8Array): Promise<boolean> {
    return secp256k1.verify(
      secp256k1.Signature.fromCompact(signature),
      sha256(message),
      this.toBytes(),
    )
  }
}
