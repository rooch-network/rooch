// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { varintByteNum } from './bytes.js'

describe('VarInt Encoding (Bitcoin CompactSize)', () => {
  it('should encode single byte values correctly', () => {
    // Test boundary values for single byte encoding
    expect(Array.from(varintByteNum(0))).toEqual([0])
    expect(Array.from(varintByteNum(252))).toEqual([252]) // 0xFC

    // 253 should use 3-byte encoding
    expect(Array.from(varintByteNum(253))).toEqual([0xfd, 253, 0])
  })

  it('should encode 3-byte values correctly', () => {
    // Test 3-byte encoding range
    expect(Array.from(varintByteNum(253))).toEqual([0xfd, 253, 0])
    expect(Array.from(varintByteNum(0xffff))).toEqual([0xfd, 0xff, 0xff])

    // 0x10000 should use 5-byte encoding
    expect(Array.from(varintByteNum(0x10000))).toEqual([0xfe, 0x00, 0x00, 0x01, 0x00])
  })

  it('should encode 5-byte values correctly', () => {
    // Test 5-byte encoding range
    expect(Array.from(varintByteNum(0x10000))).toEqual([0xfe, 0x00, 0x00, 0x01, 0x00])
    expect(Array.from(varintByteNum(0xffffffff))).toEqual([0xfe, 0xff, 0xff, 0xff, 0xff])
  })

  it('should encode 9-byte values correctly', () => {
    // Test 9-byte encoding for large values
    const largeValue = 0x100000000 // 2^32
    const result = Array.from(varintByteNum(largeValue))
    expect(result[0]).toBe(0xff)
    expect(result.length).toBe(9)
  })

  it('should match Bitcoin CompactSize standard boundaries', () => {
    // Test specific boundary values according to Bitcoin standard
    const testCases = [
      { value: 252, expectedLength: 1 }, // Last single-byte value
      { value: 253, expectedLength: 3 }, // First 3-byte value
      { value: 65535, expectedLength: 3 }, // Last 3-byte value (0xFFFF)
      { value: 65536, expectedLength: 5 }, // First 5-byte value (0x10000)
      { value: 0xffffffff, expectedLength: 5 }, // Last 5-byte value
    ]

    testCases.forEach(({ value, expectedLength }) => {
      const encoded = varintByteNum(value)
      expect(encoded.length).toBe(expectedLength)
    })
  })
})
