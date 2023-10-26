// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/**
 * Test if string is hex-string
 * @param hex hex-string
 */
export function isHex(hex: string): boolean {
  return /^0x[0-9a-f]*$/i.test(hex)
}

/**
 * Test if string is whole number (0,1,2,3...)
 */
export const isStringWholeNumber = (value: string) => /^\d+$/.test(value)
