// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import sha3 from 'js-sha3'
import { Buffer } from 'buffer'
import { OutPoint } from '../types/index.js'

export class InscribeSeed {
  private utxo: OutPoint
  private block_hash: string

  constructor(block_hash: string, utxo: OutPoint) {
    this.block_hash = block_hash
    this.utxo = utxo
  }

  seed(): string {
    const blockHashBuffer = Buffer.from(this.block_hash, 'hex')
    const txidBuffer = Buffer.from(this.utxo.txid, 'hex')
    const voutBuffer = Buffer.allocUnsafe(4)
    voutBuffer.writeUInt32LE(this.utxo.vout)

    const combinedBuffer = Buffer.concat([blockHashBuffer, txidBuffer, voutBuffer])
    const hash = sha3.sha3_256(combinedBuffer)

    return hash
  }
}
