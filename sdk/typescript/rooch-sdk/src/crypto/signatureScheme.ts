// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export const SIGNATURE_SCHEME_TO_FLAG = {
  ED25519: 0x00,
  Secp256k1: 0x01,
} as const
export const SIGNATURE_SCHEME_TO_SIZE = {
  ED25519: 32,
  Secp256k1: 33,
}

export const SIGNATURE_FLAG_TO_SCHEME = {
  0x00: 'ED25519',
  0x01: 'Secp256k1',
} as const

export type SignatureScheme = 'ED25519' | 'Secp256k1'

export type SignatureFlag = keyof typeof SIGNATURE_FLAG_TO_SCHEME
