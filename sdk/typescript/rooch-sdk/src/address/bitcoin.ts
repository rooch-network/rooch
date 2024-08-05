// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bech32, bech32m } from '@scure/base'

import { bcs } from '../bcs/index.js'
import { Bytes } from '../types/index.js'
import { blake2b, bytes, isHex, validateWitness } from '../utils/index.js'

import { Address, ROOCH_ADDRESS_LENGTH } from './address.js'
import { RoochAddress } from './rooch.js'
import { MultiChainID } from './types.js'
import { ThirdPartyAddress } from './thirdparty-address.js'
import bs58check from 'bs58check'

export enum BitcoinNetowkType {
  Bitcoin,
  /// Bitcoin's testnet network.
  Testnet,
  /// Bitcoin's signet network.
  Signet,
  /// Bitcoin's regtest network.
  Regtest,
}

enum BitcoinAddressType {
  pkh = 0,
  sh = 1,
  witness = 2,
}

const PUBKEY_ADDRESS_PREFIX_MAIN = 0    // 0x00
const PUBKEY_ADDRESS_PREFIX_TEST = 111  // 0x6f
const SCRIPT_ADDRESS_PREFIX_MAIN = 5    // 0x05
const SCRIPT_ADDRESS_PREFIX_TEST = 196  // 0xc4

export class BitcoinNetwork {
  private readonly network: BitcoinNetowkType

  constructor(network?: BitcoinNetowkType) {
    this.network = network ?? BitcoinNetowkType.Bitcoin
  }

  static fromBech32Prefix(prefix: string) {
    switch (prefix) {
      case 'bc' || 'BC':
        return new BitcoinNetwork(BitcoinNetowkType.Bitcoin)
      case 'tb' || 'TB':
        return new BitcoinNetwork(BitcoinNetowkType.Testnet)
      case 'bcrt' || 'bcrt':
        return new BitcoinNetwork(BitcoinNetowkType.Regtest)
      default:
        return undefined
    }
  }

  bech32HRP(): string {
    switch (this.network) {
      case BitcoinNetowkType.Bitcoin:
        return 'bc'
      case BitcoinNetowkType.Testnet:
        return 'tb'
      case BitcoinNetowkType.Signet:
        return 'tb'
      case BitcoinNetowkType.Regtest:
        return 'bcrt'
    }
  }
}

export class BitcoinAddress extends ThirdPartyAddress implements Address {
  private readonly bytes: Bytes
  private roochAddress: RoochAddress | undefined

  constructor(input: string, network?: BitcoinNetowkType) {
    super(input)

    if (isHex(input)) {
      this.bytes = bytes('hex', input.startsWith('0x') ? input.slice(2) : input)

      let prefixed: Uint8Array
      let version = this.bytes[1]

      switch (this.bytes[0]) {
        case BitcoinAddressType.pkh:
          prefixed = new Uint8Array(22)
          prefixed[0] = version
          prefixed[1] = this.getPubkeyAddressPrefix(network)
          prefixed.set(this.bytes.slice(2))
          this.rawAddress = bs58check.encode(prefixed)
          break
        case BitcoinAddressType.sh:
          prefixed = new Uint8Array(22)
          prefixed[0] = version
          prefixed[1] = this.getScriptAddressPrefix(network)
          prefixed.set(this.bytes.slice(2))
          this.rawAddress = bs58check.encode(prefixed)
          break
        case BitcoinAddressType.witness:
          const hrp = new BitcoinNetwork(network).bech32HRP()
          const words = bech32.toWords(Buffer.from(this.bytes.slice(2)))
          words.unshift(version)
          this.rawAddress = version === 0 ? bech32.encode(hrp, words) : bech32m.encode(hrp, words)
      }
    } else {
      let info = this.decode()
      this.bytes = this.wrapAddress(info.type, info.bytes, info.version)
    }
  }

  private getPubkeyAddressPrefix(network: BitcoinNetowkType = BitcoinNetowkType.Bitcoin): number {
    return network === BitcoinNetowkType.Bitcoin
      ? PUBKEY_ADDRESS_PREFIX_MAIN
      : PUBKEY_ADDRESS_PREFIX_TEST
  }

  private getScriptAddressPrefix(network: BitcoinNetowkType = BitcoinNetowkType.Bitcoin): number {
    return network === BitcoinNetowkType.Bitcoin
      ? SCRIPT_ADDRESS_PREFIX_MAIN
      : SCRIPT_ADDRESS_PREFIX_TEST
  }

  toBytes(): Bytes {
    return bytes('utf8', this.rawAddress)
  }

  genMultiChainAddress(): Bytes {
    return bcs.MultiChainAddress.serialize({
      multiChainId: MultiChainID.Bitcoin,
      rawAddress: this.bytes,
    }).toBytes()
  }

  genRoochAddress(): RoochAddress {
    if (!this.roochAddress) {
      this.roochAddress = new RoochAddress(blake2b(this.bytes, { dkLen: ROOCH_ADDRESS_LENGTH }))
    }

    return this.roochAddress
  }

  decode() {
    let input = this.rawAddress
    if (input.length < 14 || input.length > 74) throw new Error('Invalid address length')

    const bech32_network = (() => {
      const sep = input.lastIndexOf('1')
      const bech32Prefix = sep === -1 ? input : input.substring(0, sep)

      return BitcoinNetwork.fromBech32Prefix(bech32Prefix)
    })()

    if (bech32_network !== undefined) {
      let res
      try {
        res = bech32.decode(input)
        if (res.words[0] !== 0) throw new Error(`bech32: wrong version=${res.words[0]}`)
      } catch (_) {
        // Starting from version 1 it is decoded as bech32m
        res = bech32m.decode(input)
        if (res.words[0] === 0) throw new Error(`bech32m: wrong version=${res.words[0]}`)
      }
      const [version, ...program] = res.words
      const data = bech32.fromWords(program)
      validateWitness(version, data)
      if (version === 0 && data.length === 32)
        return {
          bytes: data,
          type: BitcoinAddressType.witness, //wsh
          version: version,
        }
      else if (version === 0 && data.length === 20)
        return {
          bytes: data,
          type: BitcoinAddressType.witness, //wpkh
          version: version,
        }
      else if (version === 1 && data.length === 32)
        return {
          bytes: data,
          type: BitcoinAddressType.witness, //tr
          version: version,
        }
      else throw new Error('Unknown witness program')
    }

    const data = bs58check.decode(input)
    if (data.length !== 21) throw new Error('Invalid base58 address')
    // Pay To Public Key Hash
    if (data[0] === 0x00) {
      return {
        bytes: data.slice(1),
        type: BitcoinAddressType.pkh,
      }
    } else if (data[0] === 0x05) {
      return { bytes: data.slice(1), type: BitcoinAddressType.sh }
    }
    throw new Error(`Invalid address prefix=${data[0]}`)
  }

  private wrapAddress(type: BitcoinAddressType, bytes: Uint8Array, version?: number): Uint8Array {
    const addr = new Uint8Array(bytes.length + 1 + (version !== undefined ? 1 : 0))
    addr.set([type])
    if (version !== undefined) {
      addr.set([version], 1)
      addr.set(bytes, 2)
    } else {
      addr.set(bytes, 1)
    }
    return addr
  }
}
