// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export * from './mnemonics.js'
export * from './signatureScheme.js'
export { PublicKey, type PublicKeyInitData } from './publickey.js'
export {
  PRIVATE_KEY_SIZE,
  LEGACY_PRIVATE_KEY_SIZE,
  ROOCH_SECRET_KEY_PREFIX,
  Keypair,
  type ParsedKeypair,
  decodeRoochSercetKey,
  encodeRoochSercetKey,
} from './keypair.js'
export { Signer } from './signer.js'

export { Authenticator, BuiltinAuthValidator, BitcoinSignMessage } from './authenticator.js'
