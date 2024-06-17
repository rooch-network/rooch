// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { fromB64 } from '@mysten/bcs'
import nacl from 'tweetnacl'

import { ROOCH_ADDRESS_LENGTH, RoochAddress } from '@/address'
import type { PublicKeyInitData } from '@/crypto'
import { PublicKey, SIGNATURE_SCHEME_TO_FLAG } from '@/crypto'
import { blake2b } from '@/utils'

const PUBLIC_KEY_SIZE = 32

/**
 * An Ed25519 public key
 */
export class Ed25519PublicKey extends PublicKey<RoochAddress> {
  static SIZE = PUBLIC_KEY_SIZE

  private readonly data: Uint8Array

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
  flag(): number {
    return SIGNATURE_SCHEME_TO_FLAG.ED25519
  }

  /**
   * Verifies that the signature is valid for the provided message
   */
  async verify(message: Uint8Array, signature: Uint8Array): Promise<boolean> {
    return nacl.sign.detached.verify(message, signature, this.toBytes())
  }

  /**
   * Return the Rooch address associated with this Ed25519 public key
   */
  toAddress(): RoochAddress {
    const tmp = new Uint8Array(PUBLIC_KEY_SIZE + 1)
    tmp.set([SIGNATURE_SCHEME_TO_FLAG.ED25519])
    tmp.set(this.toBytes(), 1)

    // Each hex char represents half a byte, hence hex address doubles the length
    return new RoochAddress(blake2b(tmp, { dkLen: 32 }).slice(0, ROOCH_ADDRESS_LENGTH * 2))
  }
}
