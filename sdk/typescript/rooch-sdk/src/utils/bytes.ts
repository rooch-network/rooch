// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Buffer } from 'buffer'
import { base16, base32, base58, base58xmr, base64, base64url, hex, utf8 } from '@scure/base'
import { Bytes } from '../types/bytes.js'

const CODERS = {
  utf8,
  hex,
  base16,
  base32,
  base64,
  base64url,
  base58,
  base58xmr,
}

export function bytesEqual(a: Uint8Array, b: Uint8Array) {
  if (a === b) return true

  if (a.length !== b.length) {
    return false
  }

  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) {
      return false
    }
  }
  return true
}

export function isBytes(a: unknown): a is Bytes {
  return (
    a instanceof Uint8Array ||
    (a != null && typeof a === 'object' && a.constructor.name === 'Uint8Array')
  )
}

export type CoderType = keyof typeof CODERS
const coderTypeError =
  'Invalid encoding type. Available types: utf8, hex, base16, base32, base64, base64url, base58, base58xmr'

export const bytesToString = (type: CoderType, bytes: Bytes): string => {
  if (!CODERS.hasOwnProperty(type)) throw new TypeError(coderTypeError)
  if (!isBytes(bytes)) throw new TypeError('bytesToString() expects Uint8Array')
  return CODERS[type].encode(bytes)
}
export const str = bytesToString

export const stringToBytes = (type: CoderType, str: string): Bytes => {
  if (!CODERS.hasOwnProperty(type)) throw new TypeError(coderTypeError)
  return CODERS[type].decode(str)
}

export function concatBytes(...arrays: Uint8Array[]): Uint8Array {
  let sum = 0
  for (let i = 0; i < arrays.length; i++) {
    const a = arrays[i]
    if (!isBytes(a)) throw new Error('Uint8Array expected')
    sum += a.length
  }
  const res = new Uint8Array(sum)
  for (let i = 0, pad = 0; i < arrays.length; i++) {
    const a = arrays[i]
    res.set(a, pad)
    pad += a.length
  }
  return res
}

export function varintByteNum(input: number): Bytes {
  if (input < 253) {
    let buf = Buffer.alloc(1)
    buf.writeUInt8(input)
    return buf
  } else if (input < 0x10000) {
    let buf = Buffer.alloc(1 + 2)
    buf.writeUInt8(253)
    buf.writeUInt16LE(input, 1)
    return buf
  } else if (input < 0x100000000) {
    let buf = Buffer.alloc(1 + 4)
    buf.writeUInt8(254)
    buf.writeUInt32LE(input, 1)
    return buf
  } else {
    let buf = Buffer.alloc(1 + 8)
    buf.writeUInt8(255)
    buf.writeInt32LE(input & -1, 1)
    buf.writeUInt32LE(Math.floor(input / 0x100000000), 5)
    return buf
  }
}

// export class ByteWriter {
//   value: Bytes
//
//   constructor(value: Bytes) {
//     this.value = value
//   }
//
//   public writeUInt8(input: number) {
//     if (input < 0) {
//       input = input >>> 0
//     }
//     let buf = Buffer.from(this.value)
//     buf.writeUInt8(input, this.value.length)
//
//     this.value = buf
//   }
//
//   public writeUInt16BE(input: number) {
//     if (input < 0) {
//       input = input >>> 0
//     }
//
//     let buf = Buffer.from(this.value)
//     buf.writeUInt16BE(input, this.value.length)
//     this.value = buf
//   }
//
//   public varintBufNum(input: number) {
//     let buf = Buffer.from([])
//     if (input < 253) {
//       buf.writeUInt8(input)
//     } else if (input < 0x10000) {
//       buf.writeUInt8(253)
//       buf.writeUInt16LE(input, 1)
//     } else if (input < 0x100000000) {
//       buf.writeUInt8(254)
//       buf.writeUInt32LE(input, 1)
//     } else {
//       buf.writeUInt8(255)
//       buf.writeInt32LE(input & -1, 1)
//       buf.writeUInt32LE(Math.floor(input / 0x100000000), 5)
//     }
//
//     this.value = concatBytes(this.value, buf)
//   }
// }

export const bytes = stringToBytes
