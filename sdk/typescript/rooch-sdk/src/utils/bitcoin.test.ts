// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { describe, it, expect } from 'vitest'
import { validateWitness } from './bitcoin.js'

describe('validateWitness', () => {
  it('should accept data with length between 2 and 40 inclusive', () => {
    const validData = new Uint8Array(20)
    expect(() => validateWitness(1, validData)).not.toThrow()
  })

  it('should accept version numbers between 0 and 16 inclusive', () => {
    const validData = new Uint8Array(20)
    for (let version = 0; version <= 16; version++) {
      expect(() => validateWitness(version, validData)).not.toThrow()
    }
  })

  it('should throw error for data length less than 2', () => {
    const invalidData = new Uint8Array(1)
    expect(() => validateWitness(1, invalidData)).toThrow('Witness: invalid length')
  })

  it('should throw error for data length greater than 40', () => {
    const invalidData = new Uint8Array(41)
    expect(() => validateWitness(1, invalidData)).toThrow('Witness: invalid length')
  })

  it('should throw error for version numbers greater than 16', () => {
    const validData = new Uint8Array(20)
    expect(() => validateWitness(17, validData)).toThrow('Witness: invalid version')
  })

  it('should throw error for version 0 with data length not equal to 20 or 32', () => {
    const invalidData = new Uint8Array(25)
    expect(() => validateWitness(0, invalidData)).toThrow('Witness: invalid length for version')
  })
})
