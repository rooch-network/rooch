// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { CIRCOM_BIGINT_N, CIRCOM_BIGINT_K } from '../constants'

// https://stackoverflow.com/a/69585881
const HEX_STRINGS = '0123456789abcdef'
const MAP_HEX = {
  0: 0,
  1: 1,
  2: 2,
  3: 3,
  4: 4,
  5: 5,
  6: 6,
  7: 7,
  8: 8,
  9: 9,
  a: 10,
  b: 11,
  c: 12,
  d: 13,
  e: 14,
  f: 15,
  A: 10,
  B: 11,
  C: 12,
  D: 13,
  E: 14,
  F: 15,
} as const

// Fast Uint8Array to hex
export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes || [])
    .map((b) => HEX_STRINGS[b >> 4] + HEX_STRINGS[b & 15])
    .join('')
}

// Mimics Buffer.from(x, 'hex') logic
// Stops on first non-hex string and returns
// https://github.com/nodejs/node/blob/v14.18.1/src/string_bytes.cc#L246-L261
export function fromHex(hexString: string): Uint8Array {
  let hexStringTrimmed: string = hexString
  if (hexString[0] === '0' && hexString[1] === 'x') {
    hexStringTrimmed = hexString.slice(2)
  }
  const bytes = new Uint8Array(Math.floor((hexStringTrimmed || '').length / 2))
  let i
  for (i = 0; i < bytes.length; i++) {
    const a = MAP_HEX[hexStringTrimmed[i * 2] as keyof typeof MAP_HEX]
    const b = MAP_HEX[hexStringTrimmed[i * 2 + 1] as keyof typeof MAP_HEX]
    if (a === undefined || b === undefined) {
      break
    }
    bytes[i] = (a << 4) | b
  }
  return i === bytes.length ? bytes : bytes.slice(0, i)
}

export function Uint8ArrayToCharArray(a: Uint8Array): string[] {
  return Array.from(a).map((x) => x.toString())
}

// Works only on 32 bit sha text lengths
export function int64toBytes(num: number): Uint8Array {
  let arr = new ArrayBuffer(8) // an Int32 takes 4 bytes
  let view = new DataView(arr)
  view.setInt32(4, num, false) // byteOffset = 0 litteEndian = false
  return new Uint8Array(arr)
}

// Works only on 32 bit sha text lengths
export function int8toBytes(num: number): Uint8Array {
  let arr = new ArrayBuffer(1) // an Int8 takes 4 bytes
  let view = new DataView(arr)
  view.setUint8(0, num) // byteOffset = 0 litteEndian = false
  return new Uint8Array(arr)
}

export function bitsToUint8(bits: string[]): Uint8Array {
  let bytes = new Uint8Array(bits.length)
  for (let i = 0; i < bits.length; i += 1) {
    bytes[i] = parseInt(bits[i], 2)
  }
  return bytes
}

export function uint8ToBits(uint8: Uint8Array): string {
  return uint8.reduce((acc, byte) => acc + byte.toString(2).padStart(8, '0'), '')
}

export function mergeUInt8Arrays(a1: Uint8Array, a2: Uint8Array): Uint8Array {
  // sum of individual array lengths
  var mergedArray = new Uint8Array(a1.length + a2.length)
  mergedArray.set(a1)
  mergedArray.set(a2, a1.length)
  return mergedArray
}

export function assert(cond: boolean, errorMessage: string) {
  if (!cond) {
    throw new Error(errorMessage)
  }
}

export function bigIntToChunkedBytes(
  // eslint-disable-next-line @typescript-eslint/ban-types
  num: BigInt | bigint,
  bytesPerChunk: number,
  numChunks: number,
) {
  const res = []
  const bigintNum: bigint = typeof num == 'bigint' ? num : num.valueOf()
  const msk = (1n << BigInt(bytesPerChunk)) - 1n
  for (let i = 0; i < numChunks; ++i) {
    res.push(((bigintNum >> BigInt(i * bytesPerChunk)) & msk).toString())
  }
  return res
}

// eslint-disable-next-line @typescript-eslint/ban-types
export function toCircomBigIntBytes(num: BigInt | bigint) {
  return bigIntToChunkedBytes(num, CIRCOM_BIGINT_N, CIRCOM_BIGINT_K)
}
