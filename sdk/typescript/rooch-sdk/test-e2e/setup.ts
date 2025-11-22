// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as fs from 'fs'
import Websocket from 'ws'
import { RoochAddress } from '../src/address/index.js'
import { getRoochNodeUrl, RoochClient, RoochHTTPTransport } from '../src/client/index.js'
import { Secp256k1Keypair } from '../src/keypairs/index.js'
import { Transaction } from '../src/transactions/index.js'
import { Args } from '../src/bcs/args.js'

import { TestBox as TestBoxA, RoochContainer } from '@roochnetwork/test-suite'

export const DEFAULT_NODE_URL = import.meta.env.VITE_FULLNODE_URL ?? getRoochNodeUrl('localnet')

export class TestBox extends TestBoxA {
  private client: RoochClient
  keypair: Secp256k1Keypair

  constructor(keypair: Secp256k1Keypair, url?: string) {
    super()
    this.keypair = keypair
    this.client = new RoochClient({
      transport: new RoochHTTPTransport({
        url: url || DEFAULT_NODE_URL,
        WebSocketConstructor: Websocket as any,
      }),
    })
    
    // Configure rooch CLI to use the same RPC endpoint
    // This is needed for commands like 'rooch move publish'
    if (url || DEFAULT_NODE_URL) {
      const rpcUrl = url || DEFAULT_NODE_URL
      this.roochCommand([
        'env',
        'add',
        '--config-dir',
        this.roochDir,
        '--alias',
        'local',
        '--rpc',
        rpcUrl,
      ])
      this.roochCommand(['env', 'switch', '--config-dir', this.roochDir, '--alias', 'local'])
    }
  }

  static setup(url?: string): TestBox {
    const kp = Secp256k1Keypair.generate()
    return new TestBox(kp, url)
  }

  async loadRoochEnv(
    target: RoochContainer | 'local' | 'container' = 'local',
    port: number = 0, // Use 0 to auto-assign available port
  ): Promise<void> {
    await super.loadRoochEnv(target, port)
    const roochServerAddress = super.getRoochServerAddress()

    this.client = new RoochClient({
      url: `http://${roochServerAddress}`,
    })
  }

  async cleanEnv() {
    // Clean up client resources
    this.client.destroy()

    // Clean up environment
    super.cleanEnv()
  }

  getClient(): RoochClient {
    return this.client
  }

  address(): RoochAddress {
    return this.keypair.getRoochAddress()
  }

  async signAndExecuteTransaction(tx: Transaction) {
    const result = await this.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: this.keypair,
    })

    return result.execution_info.status.type === 'executed'
  }

  async publishPackage(
    packagePath: string,
    box: TestBox,
    options: {
      namedAddresses: string
    } = {
      namedAddresses: 'rooch_examples=default',
    },
  ) {
    const namedAddresses = options.namedAddresses.replaceAll(
      'default',
      box.address().toHexAddress(),
    )
    this.roochCommand(
      `move build -p ${packagePath} --named-addresses ${namedAddresses} --install-dir ${this.tmpDir.name} --json`,
    )

    let fileBytes: Uint8Array
    try {
      fileBytes = fs.readFileSync(this.tmpDir.name + '/package.rpd')
      const tx = new Transaction()
      tx.callFunction({
        target: '0x2::module_store::publish_package_entry',
        args: [Args.vec('u8', Array.from(fileBytes))],
      })

      return await box.signAndExecuteTransaction(tx)
    } catch (error) {
      console.log(error)
      return false
    }
  }
}
