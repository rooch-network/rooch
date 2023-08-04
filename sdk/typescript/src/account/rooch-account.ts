// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import nacl from "tweetnacl"
import * as bip39 from "@scure/bip39"
import { bytesToHex } from "@noble/hashes/utils"
import { blake2b } from "@noble/hashes/blake2b"
import { HexString, MaybeHexString } from "../types";
import { derivePath } from "../utils"

const PUBLIC_KEY_SIZE = 32
export const ROOCH_ADDRESS_LENGTH = 32

/**
 * Class for creating and managing Aptos account
 * TODO: authKey, rotate their private key(s)
 */
export class RoochAccount {
  /**
   * A private key and public key, associated with the given account
   */
  readonly signingKey: nacl.SignKeyPair

  /**
   * Address associated with the given account
   */
  private readonly accountAddress: HexString

  /**
   * Test derive path
   */
  static isValidPath(path: string): boolean {
    return /^m\/44'\/784'\/[0-9]+'\/[0-9]+'\/[0-9]+'+$/.test(path)
  }

  /**
   * Creates new account with bip44 path and mnemonics,
   * @param path. (e.g. m/44'/637'/0'/0'/0')
   * Detailed description: {@link https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki}
   * @param mnemonics.
   * @returns AptosAccount
   */
  static fromDerivePath(path: string, mnemonics: string): RoochAccount {
    if (!RoochAccount.isValidPath(path)) {
      throw new Error("Invalid derivation path")
    }

    const normalizeMnemonics = mnemonics
      .trim()
      .split(/\s+/)
      .map((part) => part.toLowerCase())
      .join(" ")

    const { key } = derivePath(
      path,
      bytesToHex(bip39.mnemonicToSeedSync(normalizeMnemonics))
    )

    return new RoochAccount(key)
  }

  /**
   * Creates new account instance. Constructor allows passing in an address,
   * @param privateKeyBytes  Private key from which account key pair will be generated.
   * If not specified, new key pair is going to be created.
   * @param address Account address (e.g. 0xe8012714cd17606cee7188a2a365eef3fe760be598750678c8c5954eb548a591).
   * If not specified, a new one will be generated from public key
   */
  constructor(
    privateKeyBytes?: Uint8Array | undefined,
    address?: MaybeHexString
  ) {
    if (privateKeyBytes) {
      this.signingKey = nacl.sign.keyPair.fromSeed(privateKeyBytes.slice(0, 32))
    } else {
      this.signingKey = nacl.sign.keyPair()
    }
    
    const tmp = new Uint8Array(PUBLIC_KEY_SIZE+1)

    tmp.set([0x00])
    tmp.set(this.signingKey.publicKey)

    this.accountAddress = HexString.ensure(
      address ||
          bytesToHex(blake2b(tmp, { dkLen: 32 })).slice(0, ROOCH_ADDRESS_LENGTH * 2)
    )
  }

  /**
   * This is the key by which Aptos account is referenced.
   * It is the 32-byte of the SHA-3 256 cryptographic hash
   * of the public key(s) concatenated with a signature scheme identifier byte
   * @returns Address associated with the given account
   */
  address(): HexString {
    return this.accountAddress
  }

  /**
   * This key is generated with Ed25519 scheme.
   * Public key is used to check a signature of transaction, signed by given account
   * @returns The public key for the associated account
   */
  pubKey(): HexString {
    return HexString.fromUint8Array(this.signingKey.publicKey)
  }

  /**
   * Signs specified `buffer` with account's private key
   * @param buffer A buffer to sign
   * @returns A signature HexString
   */
  signBuffer(buffer: Uint8Array): HexString {
    const signature = nacl.sign.detached(buffer, this.signingKey.secretKey)
    return HexString.fromUint8Array(signature)
  }

  /**
   * Signs specified `hexString` with account's private key
   * @param hexString A regular string or HexString to sign
   * @returns A signature HexString
   */
  signHexString(hexString: MaybeHexString): HexString {
    const toSign = HexString.ensure(hexString).toUint8Array()
    return this.signBuffer(toSign)
  }

  /**
   * Verifies the signature of the message with the public key of the account
   * @param message a signed message
   * @param signature the signature of the message
   */
  verifySignature(message: MaybeHexString, signature: MaybeHexString): boolean {
    const rawMessage = HexString.ensure(message).toUint8Array()
    const rawSignature = HexString.ensure(signature).toUint8Array()
    return nacl.sign.detached.verify(
      rawMessage,
      rawSignature,
      this.signingKey.publicKey
    )
  }
}
