// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

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
export const str = bytesToString // as in python, but for bytes only

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

export const bytes = stringToBytes
