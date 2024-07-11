// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { isValidBIP32Path, isValidHardenedPath } from './mnemonics.js'

describe('isValidHardenedPath', () => {
  it('should return true for valid path with typical indices', () => {
    const path = "m/44'/784'/0'/0'/0'"
    const result = isValidHardenedPath(path)
    expect(result).toBe(true)
  })

  it('should return true for valid path with higher indices', () => {
    const path = "m/44'/784'/123'/456'/789'"
    const result = isValidHardenedPath(path)
    expect(result).toBe(true)
  })

  it('should return false for path with missing "m" prefix', () => {
    const path = "44'/784'/0'/0'/0'"
    const result = isValidHardenedPath(path)
    expect(result).toBe(false)
  })

  it('should return false for path with missing apostrophes', () => {
    const path = 'm/44/784/0/0/0'
    const result = isValidHardenedPath(path)
    expect(result).toBe(false)
  })

  it('should return false for path with non-numeric indices', () => {
    const path = "m/44'/784'/a'/b'/c'"
    const result = isValidHardenedPath(path)
    expect(result).toBe(false)
  })
})

describe('isValidBIP32Path function', () => {
  it("should return false for valid BIP32 path m/54'/784'/0'/0/0", () => {
    const path = "m/54'/784'/0'/0/0"
    expect(isValidBIP32Path(path)).toBe(false)
  })

  it("should return true for invalid BIP32 path n/54'/784'/0'/0/0", () => {
    const path = "n/54'/784'/0'/0/0"
    expect(isValidBIP32Path(path)).toBe(true)
  })

  it("should return true for invalid BIP32 path m/53'/784'/0'/0/0", () => {
    const path = "m/53'/784'/0'/0/0"
    expect(isValidBIP32Path(path)).toBe(true)
  })

  it("should return true for invalid BIP32 path m/54'/785'/0'/0/0", () => {
    const path = "m/54'/785'/0'/0/0"
    expect(isValidBIP32Path(path)).toBe(true)
  })

  it("should return false for valid BIP32 path m/74'/784'/1'/1/1", () => {
    const path = "m/74'/784'/1'/1/1"
    expect(isValidBIP32Path(path)).toBe(false)
  })
})
