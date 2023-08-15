// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { blake2b } from '@noble/hashes/blake2b'
import { bytesToHex } from '@noble/hashes/utils'

import { fromB64 } from '../../bcs'
import type { PublicKeyInitData } from '../../crypto/publickey'
import { PublicKey } from '../../crypto/publickey'
import { SIGNATURE_SCHEME_TO_FLAG } from '../../crypto/signature'
import { ROOCH_ADDRESS_LENGTH, normalizeRoochAddress } from '../../types'

const PUBLIC_KEY_SIZE = 32

/**
 * An Ed25519 public key
 */
export class Ed25519PublicKey extends PublicKey {
  static SIZE = PUBLIC_KEY_SIZE

  private data: Uint8Array

  /**
   * Create a new Ed25519PublicKey object
   * @param value ed25519 public key as buffer or base-64 encoded string
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

    if (this.data.length !== PUBLIC_KEY_SIZE) {
      throw new Error(
        `Invalid public key input. Expected ${PUBLIC_KEY_SIZE} bytes, got ${this.data.length}`,
      )
    }
  }

  /**
   * Checks if two Ed25519 public keys are equal
   */
  override equals(publicKey: Ed25519PublicKey): boolean {
    return super.equals(publicKey)
  }

  /**
   * Return the byte array representation of the Ed25519 public key
   */
  toBytes(): Uint8Array {
    return this.data
  }

  /**
   * Return the Rooch address associated with this Ed25519 public key
   */
  toRoochAddress(): string {
    const tmp = new Uint8Array(PUBLIC_KEY_SIZE + 1)
    tmp.set([SIGNATURE_SCHEME_TO_FLAG.ED25519])
    tmp.set(this.toBytes(), 1)
    // Each hex char represents half a byte, hence hex address doubles the length
    return normalizeRoochAddress(
      bytesToHex(blake2b(tmp, { dkLen: 32 })).slice(0, ROOCH_ADDRESS_LENGTH),
    )
  }

  /**
   * Return the Rooch address associated with this Ed25519 public key
   */
  // eslint-disable-next-line class-methods-use-this
  flag(): number {
    return SIGNATURE_SCHEME_TO_FLAG.ED25519
  }
}
