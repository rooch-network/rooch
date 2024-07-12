// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { BitcoinSignMessage } from './authenticator.js'

describe('BitcoinSignMessage', () => {
  it('should correctly construct with valid txData and messageInfo', () => {
    const txData = new Uint8Array([1, 2, 3, 4])
    const messageInfo = 'Test Message Info'
    const bitcoinSignMessage = new BitcoinSignMessage(txData, messageInfo)

    expect(bitcoinSignMessage.messagePrefix).toBe('\u0018Bitcoin Signed Message:\n')
    expect(bitcoinSignMessage.messageInfo).toBe('Rooch Transaction:\nTest Message Info\n')
    expect(bitcoinSignMessage.txHash).toEqual(txData)
  })

  it('should correctly generate raw message string', () => {
    const txData = new Uint8Array([1, 2, 3, 4])
    const messageInfo = 'Test Message Info'
    const bitcoinSignMessage = new BitcoinSignMessage(txData, messageInfo)

    expect(bitcoinSignMessage.txHash).toEqual(txData)
    expect(bitcoinSignMessage.raw()).toBe('Rooch Transaction:\nTest Message Info\n01020304')
  })

  it('should handle empty messageInfo gracefully', () => {
    const txData = new Uint8Array([])
    const messageInfo = ''
    const bitcoinSignMessage = new BitcoinSignMessage(txData, messageInfo)

    expect(bitcoinSignMessage.messageInfo).toBe('Rooch Transaction:\n')
    expect(bitcoinSignMessage.raw()).toBe('Rooch Transaction:\n')
  })

  it('should correctly encode message with valid txHash and messageInfo', () => {
    const txData = new Uint8Array([0x01, 0x02, 0x03, 0x04])
    const messageInfo = 'Example message info'
    const bitcoinSignMessage = new BitcoinSignMessage(txData, messageInfo)
    const encodedData = bitcoinSignMessage.encode()
    expect(encodedData).toBeInstanceOf(Uint8Array)
    expect(encodedData.length).toBeLessThanOrEqual(255)
  })

  it('should throw error when messageInfo length causes total length to exceed 255 bytes', () => {
    const txData = new Uint8Array([0x01, 0x02, 0x03, 0x04])
    const longMessageInfo = 'A'.repeat(256)
    const bitcoinSignMessage = new BitcoinSignMessage(txData, longMessageInfo)
    expect(() => bitcoinSignMessage.encode()).toThrowError(
      'message info length cannot be greater than > 302',
    )
  })
})
