// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { fromB64 } from '../b64'
import type { SerializedSignature, SignaturePubkeyPair, SignatureScheme } from './signature'
import { SIGNATURE_FLAG_TO_SCHEME } from './signature'
import { Ed25519PublicKey, Ed25519Keypair } from '../keypairs'
import type { PublicKey } from './publickey'
import type { ExportedKeypair, Keypair } from './keypair'
import { LEGACY_PRIVATE_KEY_SIZE, PRIVATE_KEY_SIZE } from './keypair'

/// Expects to parse a serialized signature by its signature scheme to a list of signature
/// and public key pairs. The list is of length 1 if it is not multisig.
export function toParsedSignaturePubkeyPair(
  serializedSignature: SerializedSignature,
): SignaturePubkeyPair[] {
  const bytes = serializedSignature
  const signatureScheme =
    SIGNATURE_FLAG_TO_SCHEME[bytes[0] as keyof typeof SIGNATURE_FLAG_TO_SCHEME]

  const SIGNATURE_SCHEME_TO_PUBLIC_KEY = {
    ED25519: Ed25519PublicKey,
  }

  const PublicKey = SIGNATURE_SCHEME_TO_PUBLIC_KEY[signatureScheme]

  const signature = bytes.slice(1, bytes.length - PublicKey.SIZE)
  const pubkeyBytes = bytes.slice(1 + signature.length)
  const pubKey = new PublicKey(pubkeyBytes)

  return [
    {
      signatureScheme,
      signature,
      pubKey,
    },
  ]
}

/// Expects to parse a single signature pubkey pair from the serialized
/// signature. Use this only if multisig is not expected.
export function toSingleSignaturePubkeyPair(
  serializedSignature: SerializedSignature,
): SignaturePubkeyPair {
  const res = toParsedSignaturePubkeyPair(serializedSignature)
  if (res.length !== 1) {
    throw Error('Expected a single signature')
  }
  return res[0]
}

export function publicKeyFromSerialized(schema: SignatureScheme, pubKey: string): PublicKey {
  if (schema === 'ED25519') {
    return new Ed25519PublicKey(pubKey)
  }
  throw new Error('Unknown public key schema')
}

export function fromExportedKeypair(keypair: ExportedKeypair): Keypair {
  const secretKey = fromB64(keypair.privateKey)
  let pureSecretKey
  switch (keypair.schema) {
    case 'ED25519':
      pureSecretKey = secretKey
      if (secretKey.length === LEGACY_PRIVATE_KEY_SIZE) {
        // This is a legacy secret key, we need to strip the public key bytes and only read the first 32 bytes
        pureSecretKey = secretKey.slice(0, PRIVATE_KEY_SIZE)
      }
      return Ed25519Keypair.fromSecretKey(pureSecretKey)
    default:
      throw new Error(`Invalid keypair schema ${keypair.schema}`)
  }
}
