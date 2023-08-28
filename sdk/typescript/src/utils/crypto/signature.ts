// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { PublicKey } from './publickey'

// TODO MultiSig
export type SignatureScheme = 'ED25519'

/**
 * Pair of signature and corresponding public key
 */
export type SignaturePubkeyPair = {
  signatureScheme: SignatureScheme
  /** Base64-encoded signature */
  signature: Uint8Array
  /** Base64-encoded public key */
  pubKey: PublicKey
}

/**
 * (`flag || signature || pubkey` bytes, as base-64 encoded string).
 * Signature is committed to the intent message of the transaction data, as base-64 encoded string.
 */
export type SerializedSignature = Uint8Array

export const SIGNATURE_SCHEME_TO_FLAG = {
  ED25519: 0x00,
}

export const SIGNATURE_FLAG_TO_SCHEME = {
  0x00: 'ED25519',
} as const

export type SignatureFlag = keyof typeof SIGNATURE_FLAG_TO_SCHEME

export function toSerializedSignature({
  signature,
  signatureScheme,
  pubKey,
}: SignaturePubkeyPair): SerializedSignature {
  const pubKeyBytes = pubKey.toBytes()
  const serializedSignature = new Uint8Array(1 + signature.length + pubKeyBytes.length)
  serializedSignature.set([SIGNATURE_SCHEME_TO_FLAG[signatureScheme]])
  serializedSignature.set(signature, 1)
  serializedSignature.set(pubKeyBytes, 1 + signature.length)
  return serializedSignature
}
