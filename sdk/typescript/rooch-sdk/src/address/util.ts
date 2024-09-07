// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bech32m } from '@scure/base'

import { isBytes, str } from '../utils/bytes.js'
import { fromHEX, getHexByteLength, isHex } from '../utils/hex.js'
import { address } from '../types/rooch.js'
import { Bytes } from '../types/bytes.js'

import { BitcoinAddress } from './bitcoin.js'
import { ROOCH_ADDRESS_LENGTH, ROOCH_BECH32_PREFIX } from './address.js'

export function decodeToRoochAddressStr(input: address): string {
  if (typeof input === 'string') {
    if (isValidRoochAddress(input)) {
      return input
    }

    if (isValidBitcoinAddress(input)) {
      return new BitcoinAddress(input).genRoochAddress().toHexAddress()
    }

    throw Error('Invalid Address')
  }

  if (isBytes(input)) {
    return str('hex', input)
  }

  return decodeToRoochAddressStr(input.toStr())
}

export function convertToRoochAddressBytes(input: address): Bytes {
  if (typeof input === 'string') {
    const normalizeAddress = normalizeRoochAddress(input)
    if (isHex(normalizeAddress) && getHexByteLength(normalizeAddress) === ROOCH_ADDRESS_LENGTH) {
      return fromHEX(normalizeAddress)
    }

    if (input.startsWith(ROOCH_BECH32_PREFIX)) {
      const decode = bech32m.decode(input)
      const bytes = bech32m.fromWords(decode.words)

      if (decode.prefix === ROOCH_BECH32_PREFIX && bytes.length === ROOCH_ADDRESS_LENGTH) {
        return bytes
      }
    }
    // throw new Error('invalid address')

    return new BitcoinAddress(input).genRoochAddress().toBytes()
  }

  return isBytes(input) ? input : convertToRoochAddressBytes(input.toStr())
}

export function isValidBitcoinAddress(input: string): boolean {
  try {
    new BitcoinAddress(input)
    return true
  } catch (_) {}

  return false
}

export function isValidRoochAddress(input: address): input is string {
  if (typeof input === 'string') {
    const normalizeAddress = normalizeRoochAddress(input)
    if (isHex(normalizeAddress) && getHexByteLength(normalizeAddress) === ROOCH_ADDRESS_LENGTH) {
      return true
    }

    if (input.startsWith(ROOCH_BECH32_PREFIX)) {
      const decode = bech32m.decode(input)
      const bytes = bech32m.fromWords(decode.words)

      return decode.prefix === ROOCH_BECH32_PREFIX && bytes.length === ROOCH_ADDRESS_LENGTH
    }

    return false
  }

  return isBytes(input) ? input.length === ROOCH_ADDRESS_LENGTH : isValidAddress(input.toStr())
}

export function isValidAddress(input: address): input is string {
  if (typeof input === 'string') {
    if (isValidRoochAddress(input)) {
      return true
    }

    return isValidBitcoinAddress(input)
  }

  return isBytes(input) ? input.length === ROOCH_ADDRESS_LENGTH : isValidAddress(input.toStr())
}

/**
 * Perform the following operations:
 * 1. Make the address lower case
 * 2. Prepend `0x` if the string does not start with `0x`.
 * 3. Add more zeros if the length of the address(excluding `0x`) is less than `ROOCH_ADDRESS_LENGTH`
 *
 * WARNING: if the address value itself starts with `0x`, e.g., `0x0x`, the default behavior
 * is to treat the first `0x` not as part of the address. The default behavior can be overridden by
 * setting `forceAdd0x` to true
 *
 */
export function normalizeRoochAddress(input: string, forceAdd0x: boolean = false): string {
  let address = input.toLowerCase()
  if (!forceAdd0x && address.startsWith('0x')) {
    address = address.slice(2)
  }
  return `0x${address.padStart(ROOCH_ADDRESS_LENGTH * 2, '0')}`
}

export function canonicalRoochAddress(input: string, forceAdd0x: boolean = false): string {
  return normalizeRoochAddress(input, forceAdd0x)
}
