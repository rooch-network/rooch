// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/* eslint-disable no-param-reassign */
export const IS_BROWSER = typeof window !== 'undefined'

const STRING_ZERO = '0'

/**
 * Some functions recreated from https://github.com/pedrouid/enc-utils/blob/master/src/index.ts
 * enc-utils is not a dependency to avoid using `Buffer` which only works in node and not browsers
 */

/**
 * Convert array buffer to string
 *
 * *[internal usage]*
 */
export function arrayBufferToString(array: ArrayBuffer): string {
  return new Uint8Array(array).reduce((data, byte) => data + String.fromCharCode(byte), '')
}

/**
 * Convert string to array buffer
 *
 * *[internal usage]*
 */
export function stringToArrayBuffer(s: string): Uint8Array {
  return Uint8Array.from(s, (c) => c.charCodeAt(0))
}

/**
 * Convert string to array buffer (browser and node compatible)
 */
export function atobUniversal(a: string): Uint8Array {
  return stringToArrayBuffer(atob(a))
}

/**
 * Convert array buffer to string (browser and node compatible)
 */
export function btoaUniversal(b: ArrayBuffer): string {
  return btoa(arrayBufferToString(b))
}

/**
 * Convert array buffer to hex-string
 * @returns format: hex-string
 */
export function buf2hex(buffer: Uint8Array) {
  return [...buffer].map((x) => x.toString(16).padStart(2, '0')).join('')
}

/**
 * Remove hex prefix '0x' from hex-string
 * @param hex hex-string
 * @returns format: base16-string
 */
export function removeHexPrefix(hex: string): string {
  return hex.replace(/^0x/i, '')
}

/**
 * Add hex prefix '0x' to base16-string
 * @param hex base16-string
 * @returns format: hex-string
 */
export function addHexPrefix(hex: string): string {
  return `0x${removeHexPrefix(hex)}`
}

/**
 * Prepend or append to string
 *
 * *[internal usage]*
 */
function padString(str: string, length: number, left: boolean, padding = STRING_ZERO): string {
  const diff = length - str.length
  let result = str
  if (diff > 0) {
    const pad = padding.repeat(diff)
    result = left ? pad + str : str + pad
  }
  return result
}

/**
 * Prepend string (default with '0')
 */
export function padLeft(str: string, length: number, padding = STRING_ZERO): string {
  return padString(str, length, true, padding)
}

/**
 * Calculate byte length of string
 *
 * *[no internal usage]*
 */
export function calcByteLength(str: string, byteSize = 8): number {
  const { length } = str
  const remainder = length % byteSize
  return remainder ? ((length - remainder) / byteSize) * byteSize + byteSize : length
}

/**
 * Prepend '0' to string bytes
 *
 * *[no internal usage]*
 */
export function sanitizeBytes(str: string, byteSize = 8, padding = STRING_ZERO): string {
  return padLeft(str, calcByteLength(str, byteSize), padding)
}

/**
 * Prepend '0' to hex-string bytes
 *
 * *[no internal usage]*
 * @param hex hex-string
 * @returns format: hex-string
 */
export function sanitizeHex(hex: string): string {
  hex = removeHexPrefix(hex)
  hex = sanitizeBytes(hex, 2)
  if (hex) {
    hex = addHexPrefix(hex)
  }
  return hex
}

/**
 * Convert utf8-string to Uint8Array
 *
 * Implemented using TextEncoder to make it isomorphic
 * @param str utf8-string
 */
export function utf8ToArray(str: string): Uint8Array {
  return new TextEncoder().encode(str)
}

/**
 * String transformation util
 *
 * Pascal case to screaming snake case
 */
export const pascalToSnake = (text: string) =>
  /[a-z]/.test(text)
    ? text
        .split(/(?=[A-Z])/)
        .join('_')
        .toUpperCase()
    : text
