// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { splitGenericParameters } from '@mysten/bcs'

import { concatBytes, isBytes, sha3_256, stringToBytes, toHEX } from '../utils/index.js'
import { canonicalRoochAddress, normalizeRoochAddress, RoochAddress } from '../address/index.js'

import { StructTag, TypeTag, BcsTypeTag } from './types.js'
import { address } from '../types/rooch.js'
import { Bytes } from '../types/bytes.js'

const VECTOR_REGEX = /^vector<(.+)>$/
const STRUCT_REGEX = /^([^:]+)::([^:]+)::([^<]+)(<(.+)>)?/

export class Serializer {
  static structTagToObjectID(input: StructTag | string): string {
    return `0x${toHEX(sha3_256(typeof input === 'string' ? input : Serializer.structTagToCanonicalString(input)))}`
  }

  static accountNamedObjectID(address: address, structTag: StructTag | string) {
    let addressBytes: Bytes
    if (typeof address === 'string') {
      const normalizeAddress = normalizeRoochAddress(address)
      addressBytes = new RoochAddress(normalizeAddress).toBytes()
    } else if (isBytes(address)) {
      addressBytes = address
    } else {
      addressBytes = address.toBytes()
    }

    const tagBytes = stringToBytes(
      'utf8',
      typeof structTag === 'string' ? structTag : Serializer.structTagToCanonicalString(structTag),
    )

    return `0x${toHEX(sha3_256(concatBytes(addressBytes, tagBytes)))}`
  }

  static structTagToCanonicalString(input: StructTag): string {
    let result = `${canonicalRoochAddress(input.address)}::${input.module}::${input.name}`

    if (input.typeParams && input.typeParams.length > 0) {
      const typeParams = input.typeParams.map(Serializer.typeTagToString).join(',')
      result += `<${typeParams}>`
    }

    return result
  }

  static typeTagToString(input: TypeTag): string {
    if (typeof input === 'string') {
      return input
    }

    if ('Vector' in input) {
      return `vector<${Serializer.typeTagToString(input.Vector)}>`
    }

    if ('Struct' in input) {
      return Serializer.structTagToCanonicalString(input.Struct)
    }

    throw new Error('Invalid TypeTag')
  }

  static typeTagParseFromStr(str: string, normalizeAddress = false): BcsTypeTag {
    if (str === 'address') {
      return { address: null }
    } else if (str === 'bool') {
      return { bool: null }
    } else if (str === 'u8') {
      return { u8: null }
    } else if (str === 'u16') {
      return { u16: null }
    } else if (str === 'u32') {
      return { u32: null }
    } else if (str === 'u64') {
      return { u64: null }
    } else if (str === 'u128') {
      return { u128: null }
    } else if (str === 'u256') {
      return { u256: null }
    } else if (str === 'signer') {
      return { signer: null }
    }

    const vectorMatch = str.match(VECTOR_REGEX)
    if (vectorMatch) {
      return {
        vector: Serializer.typeTagParseFromStr(vectorMatch[1], normalizeAddress),
      }
    }

    const structMatch = str.match(STRUCT_REGEX)
    if (structMatch) {
      const address = normalizeAddress ? normalizeRoochAddress(structMatch[1]) : structMatch[1]
      return {
        struct: {
          address,
          module: structMatch[2],
          name: structMatch[3],
          typeParams:
            structMatch[5] === undefined
              ? []
              : Serializer.parseStructTypeArgs(structMatch[5], normalizeAddress),
        },
      }
    }

    throw new Error(`Encountered unexpected token when parsing type args for ${str}`)
  }

  static parseStructTypeArgs(str: string, normalizeAddress = false): BcsTypeTag[] {
    return splitGenericParameters(str).map((tok) =>
      Serializer.typeTagParseFromStr(tok, normalizeAddress),
    )
  }

  static tagToString(tag: BcsTypeTag): string {
    if ('bool' in tag) {
      return 'bool'
    }
    if ('u8' in tag) {
      return 'u8'
    }
    if ('u16' in tag) {
      return 'u16'
    }
    if ('u32' in tag) {
      return 'u32'
    }
    if ('u64' in tag) {
      return 'u64'
    }
    if ('u128' in tag) {
      return 'u128'
    }
    if ('u256' in tag) {
      return 'u256'
    }
    if ('address' in tag) {
      return 'address'
    }
    if ('signer' in tag) {
      return 'signer'
    }
    if ('vector' in tag) {
      return `vector<${Serializer.tagToString(tag.vector)}>`
    }
    if ('struct' in tag) {
      const struct = tag.struct
      const typeParams = struct.typeParams.map(Serializer.tagToString).join(', ')
      return `${struct.address}::${struct.module}::${struct.name}${
        typeParams ? `<${typeParams}>` : ''
      }`
    }
    throw new Error('Invalid TypeTag')
  }
}
