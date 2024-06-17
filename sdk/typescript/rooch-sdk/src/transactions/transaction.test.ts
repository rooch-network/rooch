// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Transaction } from '@/transactions/transaction.ts'
import { Secp256k1Keypair } from '@/keypairs'
import { bcs } from '@/bcs'
import { BitcoinSignMessage } from '@/crypto/authenticator.ts'
import { str } from '@/utils'

describe('Transactions', () => {
  it('verify transaction', async () => {
    const signer = new Secp256k1Keypair()

    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::empty::empty_with_signer',
    })

    tx.setSender(signer.getRoochAddress().toHexAddress())
    tx.setSeqNumber(BigInt(0))
    tx.setChainId(BigInt(4))

    const auth = await signer.signTransaction(tx)

    const payload = bcs.BitcoinAuthPayload.parse(auth.payload)

    const bitcoinMessage = new BitcoinSignMessage(tx.hashData(), str('utf8', payload.messageInfo))

    const result = await signer.getPublicKey().verify(bitcoinMessage.hash(), payload.signature)
    expect(result).toBeTruthy()
  })
})
