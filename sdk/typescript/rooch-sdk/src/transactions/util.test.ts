// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { normalizeTypeArgs, normalizeTypeArgsToStr } from './util.js'

describe('normalizeTypeArgs', () => {
  it('should correctly split "target" string into three parts when "target" is present', () => {
    const input = { target: 'address::module::name' }
    const result = normalizeTypeArgs(input)
    expect(result).toEqual(['address', 'module', 'name'])
  })

  it('should return array with "address", "module", and "name" when "target" is not present', () => {
    const input = { address: 'address', module: 'module', name: 'name' }
    const result = normalizeTypeArgs(input)
    expect(result).toEqual(['address', 'module', 'name'])
  })

  it('should throw error when "target" string does not contain exactly three parts', () => {
    const input = { target: 'address::module' }
    expect(() => normalizeTypeArgs(input)).toThrow('invalid type')
  })

  it('should throw error when "target" string is empty', () => {
    const input = { target: '' }
    expect(() => normalizeTypeArgs(input)).toThrow('invalid type')
  })

  it('should return array with empty strings when "address", "module", or "name" properties are empty', () => {
    const input = { address: '', module: '', name: '' }
    const result = normalizeTypeArgs(input)
    expect(result).toEqual(['', '', ''])
  })
})

describe('normalizeTypeArgsToStr', () => {
  it('should return formatted string when input contains address, module, and name', () => {
    const input = { address: '0x1', module: 'Module', name: 'Name' }
    const result = normalizeTypeArgsToStr(input)
    expect(result).toBe('0x1::Module::Name')
  })

  it('should return target string when input contains target', () => {
    const input = { target: '0x1::Module::Name' }
    const result = normalizeTypeArgsToStr(input)
    expect(result).toBe('0x1::Module::Name')
  })

  it('should throw error when target string does not contain exactly three parts separated by "::"', () => {
    const input = { target: '0x1::Module' }
    expect(() => normalizeTypeArgsToStr(input)).toThrow('invalid type')
  })

  it('should handle empty strings for address, module, and name', () => {
    const input = { address: '', module: '', name: '' }
    const result = normalizeTypeArgsToStr(input)
    expect(result).toBe('::::')
  })

  it('should throw error when target is an empty string', () => {
    const input = { target: '' }
    expect(() => normalizeTypeArgsToStr(input)).toThrow('invalid type')
  })
})
