// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/bytes.js'

export function isHex(input: string | Bytes): boolean {
  if (typeof input === 'string') {
    return /^(0x|0X)?[a-fA-F0-9]+$/.test(input) && input.length % 2 === 0
  } else {
    for (let i = 0; i < input.length; i++) {
      const byte = input[i]
      // Check if the byte is a valid hex character (0-9, A-F, a-f)
      if (
        !((byte >= 48 && byte <= 57) || (byte >= 65 && byte <= 70) || (byte >= 97 && byte <= 102))
      ) {
        return false
      }
    }
    return true
  }
}

export function getHexByteLength(input: string): number {
  return /^(0x|0X)/.test(input) ? (input.length - 2) / 2 : input.length / 2
}

export function normalizeHex(input: string): string {
  return input.startsWith('0x') ? input.slice(2) : input
}

export function fromHEX(input: string): Bytes {
  const normalized = normalizeHex(input)
  const padded = normalized.length % 2 === 0 ? normalized : `0${normalized}}`
  const intArr = padded.match(/.{2}/g)?.map((byte) => parseInt(byte, 16)) ?? []

  return Uint8Array.from(intArr)
}

const u8a = (a: any): a is Uint8Array => a instanceof Uint8Array

const hexes = Array.from({ length: 256 }, (_, i) => i.toString(16).padStart(2, '0'))

export function toHEX(input: Bytes): string {
  if (!u8a(input)) throw new Error('Uint8Array expected')
  // pre-caching improves the speed 6x
  let hex = ''
  for (let i = 0; i < input.length; i++) {
    hex += hexes[input[i]]
  }
  return hex
}
