// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export function toHexString(byteArray: Iterable<number>): string {
  const hexArray = Array.from(byteArray).map((byte) => {
    const roundedByte = Math.floor(byte)
    return (roundedByte < 0 ? 256 + roundedByte : roundedByte).toString(16).padStart(2, '0')
  })
  return `0x${hexArray.join('')}`
}

export function fromHexString(hex: string, padding?: number): Uint8Array {
  let hexWithoutPrefix = hex.startsWith('0x') ? hex.substring(2) : hex

  if (padding && hexWithoutPrefix.length < padding) {
    hexWithoutPrefix = padLeft(hexWithoutPrefix, padding)
  } else if (!padding && hexWithoutPrefix.length % 2 !== 0) {
    hexWithoutPrefix = `0${hexWithoutPrefix}`
  }

  const byteArray = new Uint8Array(hexWithoutPrefix.length / 2)

  for (let i = 0; i < hexWithoutPrefix.length; i += 2) {
    byteArray[i / 2] = parseInt(hexWithoutPrefix.substring(i, i + 2), 16)
  }

  return byteArray
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
