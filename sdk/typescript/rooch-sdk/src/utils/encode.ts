// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { hexlify } from '@ethersproject/bytes'
import { sha3_256 } from '@noble/hashes/sha3'
import { ROOCH_ADDRESS_LENGTH } from '../constants'
import { FunctionId, AccountAddress, Identifier, TypeTag, StructTag } from '../types'

export function functionIdToStirng(functionId: FunctionId): string {
  if (typeof functionId !== 'string') {
    if (functionId instanceof Object) {
      return `${functionId.address}::${functionId.module}::${functionId.functionName}`
    }
  }
  return functionId
}

export function parseFunctionId(functionId: FunctionId): {
  address: AccountAddress
  module: Identifier
  functionName: Identifier
} {
  if (typeof functionId !== 'string') {
    return functionId
  }
  const parts = functionId.split('::', 3)

  if (parts.length !== 3) {
    throw new Error(`cannot parse ${functionId} into FunctionId`)
  }

  return {
    address: normalizeRoochAddress(parts[0]),
    module: parts[1],
    functionName: parts[2],
  }
}

/**
 * Perform the following operations:
 * 1. Make the address lower case
 * 2. Prepend `0x` if the string does not start with `0x`.
 * 3. Add more zeros if the length of the address(excluding `0x`) is less than `Rooch_ADDRESS_LENGTH`
 *
 * WARNING: if the address value itself starts with `0x`, e.g., `0x0x`, the default behavior
 * is to treat the first `0x` not as part of the address. The default behavior can be overridden by
 * setting `forceAdd0x` to true
 *
 */
export function normalizeRoochAddress(value: string, forceAdd0x: boolean = false): string {
  let address = value.toLowerCase()
  if (!forceAdd0x && address.startsWith('0x')) {
    address = address.slice(2)
  }
  return `0x${address.padStart(ROOCH_ADDRESS_LENGTH, '0')}`
}

export function canonicalRoochAddress(value: string, forceAdd0x: boolean = false): string {
  let address = value.toLowerCase()
  if (!forceAdd0x && address.startsWith('0x')) {
    address = address.slice(2)
  }

  return `${address.padStart(ROOCH_ADDRESS_LENGTH, '0')}`
}

export function typeTagToString(type_tag: TypeTag): string {
  if (typeof type_tag === 'string') {
    return type_tag
  }

  if ('Vector' in type_tag) {
    return `Vector<${typeTagToString(type_tag.Vector)}>`
  }

  if ('Struct' in type_tag) {
    const struct = type_tag.Struct
    let result = `${struct.address}::${struct.module}::${struct.name}`
    if (struct.type_params) {
      const params = struct.type_params.map(typeTagToString).join(', ')
      result += `<${params}>`
    }
    return result
  }

  throw new Error(`Unknown type tag: ${JSON.stringify(type_tag)}`)
}

export function structTagToCanonicalString(structTag: StructTag): string {
  let result = `${canonicalRoochAddress(structTag.address)}::${structTag.module}::${structTag.name}`

  if (structTag.type_params && structTag.type_params.length > 0) {
    const typeParams = structTag.type_params.map(typeTagToCanonicalString).join(',')
    result += `<${typeParams}>`
  }

  return result
}

export function typeTagToCanonicalString(typeTag: TypeTag): string {
  if (typeof typeTag === 'string') {
    return typeTag.toLocaleLowerCase()
  } else if ('Vector' in typeTag) {
    return `vector<${typeTagToCanonicalString(typeTag.Vector)}>`
  } else if ('Struct' in typeTag) {
    return structTagToCanonicalString(typeTag.Struct)
  } else {
    throw new Error(`Unknown TypeTag: ${JSON.stringify(typeTag)}`)
  }
}

export function structTagToObjectID(structTag: StructTag): string {
  const canonicalString = structTagToCanonicalString(structTag)
  const hash = sha3_256(canonicalString)
  return hexlify(hash)
}
