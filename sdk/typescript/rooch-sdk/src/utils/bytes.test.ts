// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { describe, it, expect } from 'vitest'
import {
  bytesEqual,
  bytesToString,
  CoderType,
  concatBytes,
  isBytes,
  stringToBytes,
} from './bytes.js'
import { Bytes } from '../types/bytes.js'

describe('bytesEqual', () => {
  it('should return true for identical Uint8Array instances', () => {
    const arr1 = new Uint8Array([1, 2, 3])
    const arr2 = new Uint8Array([1, 2, 3])
    expect(bytesEqual(arr1, arr2)).toBeTruthy()
  })

  it('should return false for Uint8Array instances of different lengths', () => {
    const arr1 = new Uint8Array([1, 2, 3])
    const arr2 = new Uint8Array([1, 2, 3, 4])
    expect(bytesEqual(arr1, arr2)).toBeFalsy()
  })

  it('should handle Uint8Array instances with only one element', () => {
    const arr1 = new Uint8Array([1])
    const arr2 = new Uint8Array([1])
    expect(bytesEqual(arr1, arr2)).toBeTruthy()
  })

  it('should handle Uint8Array instances with maximum possible length', () => {
    const maxSize = 2 ** 32 - 1
    const arr1 = new Uint8Array(maxSize).fill(1)
    const arr2 = new Uint8Array(maxSize).fill(1)
    expect(bytesEqual(arr1, arr2)).toBeTruthy()
  })
})

describe('isBytes', () => {
  it('should return true when given a Uint8Array instance', () => {
    const input = new Uint8Array([1, 2, 3])
    const result = isBytes(input)
    expect(result).toBe(true)
  })

  it('should return true when given an object with Uint8Array constructor name', () => {
    const input = { constructor: { name: 'Uint8Array' } }
    const result = isBytes(input)
    expect(result).toBe(true)
  })

  it('should return true when given a Uint8Array subclass instance', () => {
    class SubUint8Array extends Uint8Array {}
    const input = new SubUint8Array([1, 2, 3])
    const result = isBytes(input)
    expect(result).toBe(true)
  })
})

describe('bytesToString', () => {
  it('should correctly encode bytes to string of different types', () => {
    const input = new Uint8Array([72, 101, 108, 108, 111])
    expect(bytesToString('utf8', input)).toBe('Hello')
    expect(bytesToString('hex', input)).toBe('48656c6c6f')
    expect(bytesToString('base16', input)).toBe('48656C6C6F')
    expect(bytesToString('base32', input)).toBe('JBSWY3DP')
    expect(bytesToString('base64', input)).toBe('SGVsbG8=')
    expect(bytesToString('base64url', input)).toBe('SGVsbG8=')
    expect(bytesToString('base58', input)).toBe('9Ajdvzr')
    expect(bytesToString('base58xmr', input)).toBe('9Ajdvzr')
  })

  it('should throw error for invalid encoding type', () => {
    const input = new Uint8Array([72, 101, 108, 108, 111])
    expect(() => bytesToString('invalidType' as CoderType, input)).toThrow(TypeError)
  })

  it('should throw error when bytes is not a Uint8Array', () => {
    const input = [72, 101, 108, 108, 111]
    expect(() => bytesToString('utf8', input as unknown as Bytes)).toThrow(TypeError)
  })

  it('should handle empty Uint8Array input', () => {
    const input = new Uint8Array([])
    expect(bytesToString('utf8', input)).toBe('')
  })
})

describe('stringToBytes', () => {
  it('should correctly decode string to bytes of different types', () => {
    expect(stringToBytes('utf8', 'Hello')).toEqual(new Uint8Array([72, 101, 108, 108, 111]))
    expect(stringToBytes('hex', '48656c6c6f')).toEqual(new Uint8Array([72, 101, 108, 108, 111]))
    expect(stringToBytes('base16', '48656C6C6F')).toEqual(new Uint8Array([72, 101, 108, 108, 111]))
    expect(stringToBytes('base32', 'JBSWY3DP')).toEqual(new Uint8Array([72, 101, 108, 108, 111]))
    expect(stringToBytes('base64', 'SGVsbG8=')).toEqual(new Uint8Array([72, 101, 108, 108, 111]))
    expect(stringToBytes('base64url', 'SGVsbG8=')).toEqual(new Uint8Array([72, 101, 108, 108, 111]))
    expect(stringToBytes('base58', '9Ajdvzr')).toEqual(new Uint8Array([72, 101, 108, 108, 111]))
    expect(stringToBytes('base58xmr', '9Ajdvzr')).toEqual(new Uint8Array([72, 101, 108, 108, 111]))
  })

  it('should throw TypeError for an invalid encoding type', () => {
    const input = 'hello'
    expect(() => stringToBytes('invalidType' as CoderType, input)).toThrow(TypeError)
  })

  it('should throw error for an invalid hex string', () => {
    const input = '68656c6c6g'
    expect(() => stringToBytes('hex', input)).toThrow()
  })
})

describe('concatBytes', () => {
  it('should concatenate multiple Uint8Array instances correctly', () => {
    const arr1 = new Uint8Array([1, 2, 3])
    const arr2 = new Uint8Array([4, 5, 6])
    const result = concatBytes(arr1, arr2)
    expect(result).toEqual(new Uint8Array([1, 2, 3, 4, 5, 6]))
  })

  it('should throw error when any input is not a Uint8Array', () => {
    const arr1 = new Uint8Array([1, 2, 3])
    const arr2 = [4, 5, 6]
    expect(() => concatBytes(arr1, arr2 as any)).toThrow('Uint8Array expected')
  })

  it('should handle concatenation of empty Uint8Array instances', () => {
    const arr1 = new Uint8Array([])
    const arr2 = new Uint8Array([4, 5, 6])
    const result = concatBytes(arr1, arr2)
    expect(result).toEqual(new Uint8Array([4, 5, 6]))
  })

  it('should handle large Uint8Array instances without performance degradation', () => {
    const largeArr1 = new Uint8Array(10 ** 6).fill(1)
    const largeArr2 = new Uint8Array(10 ** 6).fill(2)
    const result = concatBytes(largeArr1, largeArr2)
    expect(result.length).toBe(2 * 10 ** 6)
    expect(result[0]).toBe(1)
    expect(result[10 ** 6]).toBe(2)
  })
})
