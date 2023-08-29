// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Buffer } from 'buffer'

export function toHexString(byteArray: Iterable<number>): string {
  return `0x${Buffer.from(new Uint8Array(byteArray)).toString('hex')}`
}

export function fromHexString(hex: string, padding?: number): Uint8Array {
  let hexWithoutPrefix = hex.startsWith('0x') ? hex.substring(2) : hex

  if (padding && hexWithoutPrefix.length < padding) {
    hexWithoutPrefix = padLeft(hexWithoutPrefix, padding)
  } else if (!padding && hexWithoutPrefix.length % 2 !== 0) {
    hexWithoutPrefix = `0${hexWithoutPrefix}`
  }

  const buf = Buffer.from(hexWithoutPrefix, 'hex')
  return new Uint8Array(buf)
}

/**
 * @public
 * Should be called to pad string to expected length
 */
export function padLeft(str: string, chars: number, sign: string = '0') {
  return new Array(chars - str.length + 1).join(sign) + str
}

/**
 * @public
 * Should be called to pad string to expected length
 */
export function padRight(str: string, chars: number, sign: string = '0') {
  return str + new Array(chars - str.length + 1).join(sign)
}
