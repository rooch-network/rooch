// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as fs from 'fs'
import tmp from 'tmp'
import { RoochAddress } from '../src/address/index.js'
import { getRoochNodeUrl, RoochClient } from '../src/client/index.js'
import { Secp256k1Keypair } from '../src/keypairs/index.js'
import { Transaction } from '../src/transactions/index.js'
import { Args } from '../src/bcs/args.js'

import { TestBox as TestBoxA, RoochContainer } from '@roochnetwork/test-suite'

export const DEFAULT_NODE_URL = import.meta.env.VITE_FULLNODE_URL ?? getRoochNodeUrl('localnet')

export class TestBox extends TestBoxA {
  private client?: RoochClient
  keypair: Secp256k1Keypair

  constructor(keypair: Secp256k1Keypair) {
    super()
    this.keypair = keypair
  }

  static setup(): TestBox {
    const kp = Secp256k1Keypair.generate()
    return new TestBox(kp)
  }

  async loadRoochEnv(
    target: RoochContainer | 'local' | 'container' = 'local',
    port: number = 6768,
  ): Promise<void> {
    await super.loadRoochEnv(target, port)
    const roochServerAddress = super.getRoochServerAddress()

    this.client = new RoochClient({
      url: `http://${roochServerAddress}`,
    })
    return
  }

  getClient(url = DEFAULT_NODE_URL): RoochClient {
    if (url === DEFAULT_NODE_URL) {
      if (!this.client) {
        this.client = new RoochClient({
          url,
        })
      }
      return this.client
    }

    return new RoochClient({
      url,
    })
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
    tmp.setGracefulCleanup()

    const tmpDir = tmp.dirSync({ unsafeCleanup: true })
    const namedAddresses = options.namedAddresses.replaceAll(
      'default',
      box.address().toHexAddress(),
    )
    this.roochCommand(
      `move build -p ${packagePath} --named-addresses ${namedAddresses} --install-dir ${tmpDir.name} --export --json`,
    )

    let fileBytes: Uint8Array
    try {
      fileBytes = fs.readFileSync(tmpDir.name + '/package.blob')
      const tx = new Transaction()
      tx.callFunction({
        target: '0x2::module_store::publish_modules_entry',
        args: [Args.vec('u8', Array.from(fileBytes))],
      })

      return await box.signAndExecuteTransaction(tx)
    } catch (error) {
      console.log(error)
      return false
    }
  }
}
