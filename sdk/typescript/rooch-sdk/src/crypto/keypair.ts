// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bech32 } from 'bech32'

import { Signer } from './signer.js'
import type { SignatureScheme } from './signatureScheme.js'
import { SIGNATURE_FLAG_TO_SCHEME, SIGNATURE_SCHEME_TO_FLAG } from './signatureScheme.js'

export const PRIVATE_KEY_SIZE = 32
export const LEGACY_PRIVATE_KEY_SIZE = 64

export const ROOCH_SECRET_KEY_PREFIX = 'roochsecretkey'

export type ParsedKeypair = {
  /**
   * The signature scheme of the private key.
   * @deprecated Use `scheme` instead. This alias is kept for backward-compatibility and will be removed in a future release.
   */
  schema?: SignatureScheme
  /**
   * The signature scheme of the private key.
   */
  scheme: SignatureScheme
  secretKey: Uint8Array
}

export abstract class Keypair extends Signer {
  /**
   * This returns the Bech32 secret key string for this keypair.
   */
  abstract getSecretKey(): string
}

/**
 * Decode a Bech32-encoded Rooch secret key.
 *
 * @param value Bech32 string starting with `roochsecretkey`.
 * @returns ParsedKeypair object containing the secret key bytes and signature scheme.
 */
export function decodeRoochSecretKey(value: string): ParsedKeypair {
  const { prefix, words } = bech32.decode(value)
  if (prefix !== ROOCH_SECRET_KEY_PREFIX) {
    throw new Error('invalid private key prefix')
  }
  const extendedSecretKey = new Uint8Array(bech32.fromWords(words))
  const secretKey = extendedSecretKey.slice(1)
  const signatureScheme =
    SIGNATURE_FLAG_TO_SCHEME[extendedSecretKey[0] as keyof typeof SIGNATURE_FLAG_TO_SCHEME]
  return {
    scheme: signatureScheme,
    schema: signatureScheme, // backward compatibility
    secretKey,
  }
}

/**
 * @deprecated Use {@link decodeRoochSecretKey} instead.
 */
export const decodeRoochSercetKey = decodeRoochSecretKey

/**
 * This returns a Bech32 encoded string starting with `roochsecretkey`,
 * encoding 33-byte `flag || bytes` for the given the 32-byte private
 * key and its signature scheme.
 */
export function encodeRoochSecretKey(bytes: Uint8Array, scheme: SignatureScheme): string {
  if (bytes.length !== PRIVATE_KEY_SIZE) {
    throw new Error('Invalid bytes length')
  }
  const flag = SIGNATURE_SCHEME_TO_FLAG[scheme]
  const privKeyBytes = new Uint8Array(bytes.length + 1)
  privKeyBytes.set([flag])
  privKeyBytes.set(bytes, 1)
  return bech32.encode(ROOCH_SECRET_KEY_PREFIX, bech32.toWords(privKeyBytes))
}

/**
 * @deprecated Use {@link encodeRoochSecretKey} instead.
 */
export const encodeRoochSercetKey = encodeRoochSecretKey
