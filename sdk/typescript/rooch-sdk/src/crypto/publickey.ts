// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'
import { Address } from '../address/index.js'
import { bytesEqual, toB64 } from '../utils/index.js'

/**
 * Value to be converted into public key.
 */
export type PublicKeyInitData = string | Bytes | Iterable<number>

/**
 * A public key
 */
export abstract class PublicKey<T extends Address> {
  /**
   * Checks if two public keys are equal
   */
  equals(publicKey: PublicKey<T>) {
    return bytesEqual(this.toBytes(), publicKey.toBytes())
  }

  /**
   * Return the base-64 representation of the public key
   */
  toBase64() {
    return toB64(this.toBytes())
  }

  toString(): string {
    throw new Error(
      '`toString` is not implemented on public keys. Use `toBase64()` or `toBytes()` instead.',
    )
  }

  /**
   * Return the byte array representation of the public key
   */
  abstract toBytes(): Uint8Array

  /**
   * Return signature scheme flag of the public key
   */
  abstract flag(): number

  abstract toAddress(): T

  /**
   * Verifies that the signature is valid  for the provided message
   */
  abstract verify(data: Uint8Array, signature: Uint8Array | string): Promise<boolean>
}
