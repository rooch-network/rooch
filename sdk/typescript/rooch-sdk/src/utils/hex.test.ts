// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { fromHEX, getHexByteLength, isHex, normalizeHex, toHEX } from './hex'

describe('isHex', () => {
  it('should return true when input is a valid hex string with "0x" prefix', () => {
    const input = '0x1a2b3c'
    const result = isHex(input)
    expect(result).toBe(true)
  })

  it('should return true when input is a valid hex string without "0x" prefix', () => {
    const input = '1a2b3c'
    const result = isHex(input)
    expect(result).toBe(true)
  })

  it('should return false when input is a string with odd length', () => {
    const input = '1a2b3'
    const result = isHex(input)
    expect(result).toBe(false)
  })

  it('should return false when input is a string with invalid characters', () => {
    const input = '1a2b3g'
    const result = isHex(input)
    expect(result).toBe(false)
  })

  it('should return false when input is a Bytes array with values outside hex range', () => {
    const input = new Uint8Array([0, 255, 256])
    const result = isHex(input)
    expect(result).toBe(false)
  })
})

describe('getHexByteLength', () => {
  it('should return correct length when input has "0x" prefix', () => {
    const input = '0x1234'
    const result = getHexByteLength(input)
    expect(result).toBe(2)
  })

  it('should handle input string with odd number of characters', () => {
    const input = '123'
    const result = getHexByteLength(input)
    expect(result).toBe(1.5)
  })

  it('should return correct length when input has "0X" prefix', () => {
    const input = '0X12G4'
    const result = getHexByteLength(input)
    expect(result).toBe(2)
  })

  it('should return 0 for empty input string', () => {
    const input = ''
    const result = getHexByteLength(input)
    expect(result).toBe(0)
  })
})

describe('normalizeHex', () => {
  it('should return string without "0x" when input starts with "0x"', () => {
    const input = '0x1a2b3c'
    const result = normalizeHex(input)
    expect(result).toBe('1a2b3c')
  })

  it('should return the same string when input does not start with "0x"', () => {
    const input = '1a2b3c'
    const result = normalizeHex(input)
    expect(result).toBe(input)
  })

  it('should return the same single character string', () => {
    const input = 'a'
    const result = normalizeHex(input)
    expect(result).toBe(input)
  })

  it('should return an empty string when input is "0x"', () => {
    const input = '0x'
    const result = normalizeHex(input)
    expect(result).toBe('')
  })

  it('should return the same string with special characters', () => {
    const input = '@#$%^&*'
    const result = normalizeHex(input)
    expect(result).toBe(input)
  })
})

describe('fromHEX', () => {
  it('should convert valid hex string with even length to Bytes', () => {
    const input = '4a6f686e'
    const expected = new Uint8Array([74, 111, 104, 110])
    const result = fromHEX(input)
    expect(result).toEqual(expected)
  })

  it('should convert valid hex string with odd length to Bytes', () => {
    const input = 'a3f'
    const expected = new Uint8Array([10, 63])
    const result = fromHEX(input)
    expect(result).toEqual(expected)
  })

  it('should handle hex strings with non-hex characters gracefully', () => {
    const input = 'zxy123'
    const result = fromHEX(input)
    expect(result).toEqual(Uint8Array.from([0, 0, 35]))
  })

  it('should process very large hex strings efficiently', () => {
    const input = 'a'.repeat(1000000)
    const result = fromHEX(input)
    expect(result.length).toBe(500000)
  })

  it('should manage hex strings with only one character', () => {
    const input = 'f'
    const expected = new Uint8Array([15])
    const result = fromHEX(input)
    expect(result).toEqual(expected)
  })
})

describe('toHEX', () => {
  it('should convert a Uint8Array to a hex string correctly', () => {
    const input = new Uint8Array([0, 1, 2, 255])
    const expectedOutput = '000102ff'
    expect(toHEX(input)).toBe(expectedOutput)
  })

  it('should return an empty string when input is an empty Uint8Array', () => {
    const input = new Uint8Array([])
    const expectedOutput = ''
    expect(toHEX(input)).toBe(expectedOutput)
  })

  it('should throw an error if input is not a Uint8Array', () => {
    const input = [0, 1, 2, 255]
    expect(() => toHEX(input as any)).toThrow('Uint8Array expected')
  })

  it('should handle Uint8Array with maximum byte values (255)', () => {
    const input = new Uint8Array([255, 255, 255])
    const expectedOutput = 'ffffff'
    expect(toHEX(input)).toBe(expectedOutput)
  })

  it('should handle Uint8Array with minimum byte values (0)', () => {
    const input = new Uint8Array([0, 0, 0])
    const expectedOutput = '000000'
    expect(toHEX(input)).toBe(expectedOutput)
  })
})
