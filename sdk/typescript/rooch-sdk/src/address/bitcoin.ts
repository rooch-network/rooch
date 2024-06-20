// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bech32, bech32m, createBase58check } from '@scure/base'

import { bcs } from '@/bcs'
import { Bytes } from '@/types'
import { blake2b, bytes, sha256, validateWitness } from '@/utils'

import { Address, ROOCH_ADDRESS_LENGTH } from './address'
import { RoochAddress } from './rooch'
import { MultiChainID } from './types'

const base58check = createBase58check(sha256)

enum BitcoinNetowk {
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
  wpkh = 2,
  wsh = 2,
  tr = 2,
}

export class BitcoinAddress implements Address {
  private readonly rawAddress: string
  private readonly bytes: Bytes
  private roochAddress: RoochAddress | undefined

  constructor(input: string) {
    let info = this.decode(input)
    this.rawAddress = input
    this.bytes = this.wrapAddress(info.type, info.bytes, info.version)
  }

  toStr(): string {
    return this.rawAddress
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

  private decode(input: string) {
    if (input.length < 14 || input.length > 74) throw new Error('Invalid address length')

    const bech32_network = (() => {
      const sep = input.lastIndexOf('1')
      const bech32Prefix = sep === -1 ? input : input.substring(0, sep)

      switch (bech32Prefix) {
        case 'bc' || 'BC':
          return BitcoinNetowk.Bitcoin
        case 'tb' || 'TB':
          return BitcoinNetowk.Testnet
        case 'bcrt' || 'bcrt':
          return BitcoinNetowk.Regtest
        default:
          return undefined
      }
    })()

    if (bech32_network) {
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
          type: BitcoinAddressType.wsh,
          version: version,
        }
      else if (version === 0 && data.length === 20)
        return {
          bytes: data,
          type: BitcoinAddressType.wpkh,
          version: version,
        }
      else if (version === 1 && data.length === 32)
        return {
          bytes: data,
          type: BitcoinAddressType.tr,
          version: version,
        }
      else throw new Error('Unknown witness program')
    }

    const data = base58check.decode(input)
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
    if (version) {
      addr.set([version], 1)
      addr.set(bytes, 2)
    } else {
      addr.set(bytes, 1)
    }
    return addr
  }
}
