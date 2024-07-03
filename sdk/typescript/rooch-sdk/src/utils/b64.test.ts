// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { describe, it, expect } from 'vitest'
import { toB64 } from './b64'

describe('toB64', () => {
  it('should convert small Uint8Array to base64 string correctly', () => {
    const input = new Uint8Array([72, 101, 108, 108, 111]) // "Hello"
    const expectedOutput = 'SGVsbG8='
    expect(toB64(input)).toBe(expectedOutput)
  })

  it('should return an empty string when input is an empty Uint8Array', () => {
    const input = new Uint8Array([])
    const expectedOutput = ''
    expect(toB64(input)).toBe(expectedOutput)
  })

  it('should process Uint8Array with length just below CHUNK_SIZE correctly', () => {
    const input = new Uint8Array(8191).fill(65) // "A" repeated 8191 times
    const expectedOutput = btoa(String.fromCharCode(...input))
    expect(toB64(input)).toBe(expectedOutput)
  })

  it('should handle Uint8Array with non-ASCII characters correctly', () => {
    const input = new Uint8Array([195, 164, 195, 182, 195, 188]) // "äöü" in UTF-8
    const expectedOutput = 'w6TDtsO8'
    expect(toB64(input)).toBe(expectedOutput)
  })
})
