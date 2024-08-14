// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import tmp, { DirResult } from 'tmp'
import { execSync } from 'child_process'
import { Network, StartedNetwork } from 'testcontainers'

import { RoochAddress } from '../src/address/index.js'
import { getRoochNodeUrl, RoochClient } from '../src/client/index.js'
import { Secp256k1Keypair } from '../src/keypairs/index.js'
import { Transaction } from '../src/transactions/index.js'
import { Args } from '../src/bcs/args.js'
import * as fs from 'fs'
import { BitcoinContainer, StartedBitcoinContainer } from './env/bitcoin-container.js'
import { RoochContainer, StartedRoochContainer } from './env/rooch-container.js'
// import os from 'node:os'
import { OrdContainer, StartedOrdContainer } from './env/ord-container.js'

const bitcoinNetworkAlias = 'bitcoind'
export const DEFAULT_NODE_URL = import.meta.env.VITE_FULLNODE_URL ?? getRoochNodeUrl('localnet')

let _defaultCmdAddress = ''

export class TestBox {
  private client?: RoochClient
  keypair: Secp256k1Keypair
  network?: StartedNetwork

  tmpDir: DirResult
  ordContainer?: StartedOrdContainer
  bitcoinContainer?: StartedBitcoinContainer
  roochContainer?: StartedRoochContainer | number

  constructor(keypair: Secp256k1Keypair) {
    this.keypair = keypair
    tmp.setGracefulCleanup()
    this.tmpDir = tmp.dirSync({ unsafeCleanup: true })
  }

  static setup(): TestBox {
    const kp = Secp256k1Keypair.generate()
    return new TestBox(kp)
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

  async loadBitcoinEnv(customContainer?: BitcoinContainer) {
    if (customContainer) {
      this.bitcoinContainer = await customContainer.start()
      return
    }

    this.bitcoinContainer = await new BitcoinContainer()
      .withHostDataPath(this.tmpDir.name)
      .withNetwork(await this.getNetwork())
      .withNetworkAliases(bitcoinNetworkAlias)
      .start()
  }

  async loadRoochEnv(customContainer?: RoochContainer) {
    if (customContainer) {
      await customContainer.start()
      return
    }

    // The container test in the linux environment is incomplete, so use it first
    // if (os.platform() === 'darwin') {
    const port = '6768'

    const cmds = ['server', 'start', '-n', 'local', '-d', 'TMP', '--port', port]

    if (this.bitcoinContainer) {
      cmds.push(
        ...[
          '--btc-rpc-url',
          'http://127.0.0.1:18443',
          '--btc-rpc-username',
          this.bitcoinContainer.getRpcUser(),
          '--btc-rpc-password',
          this.bitcoinContainer.getRpcPass(),
        ],
      )
    }

    cmds.push(`> ${this.tmpDir.name}/rooch.log 2>&1 & echo $!`)

    const result = this.roochCommand(cmds)
    this.roochContainer = parseInt(result.toString().trim(), 10)

    await this.delay(10)

    this.client = new RoochClient({ url: `http://127.0.0.1:${port}` })

    //   return
    // }

    // const container = new RoochContainer().withHostConfigPath(`${this.tmpDir.name}/.rooch`)
    // await container.initializeRooch()
    //
    // container
    //   .withNetwork(await this.getNetwork())
    //   .withNetworkName('local')
    //   .withDataDir('TMP')
    //   .withPort(6768)
    //
    // if (this.bitcoinContainer) {
    //   container
    //     .withBtcRpcUrl(`http://${bitcoinNetworkAlias}:18443`)
    //     .withBtcRpcUsername(this.bitcoinContainer.getRpcUser())
    //     .withBtcRpcPassword(this.bitcoinContainer.getRpcPass())
    //     .withBtcSyncBlockInterval(1) // Set sync interval to 1 second
    // }
    //
    // this.roochContainer = await container.start()
    //
    // this.client = new RoochClient({ url: this.roochContainer.getConnectionAddress() })
  }

  async loadORDEnv(customContainer?: OrdContainer) {
    if (customContainer) {
      this.ordContainer = await customContainer.start()
      return
    }

    if (!this.bitcoinContainer) {
      console.log('bitcoin container not init')
      return
    }

    this.ordContainer = await new OrdContainer()
      .withHostDataPath(this.tmpDir.name)
      .withBitcoinDataPath(this.bitcoinContainer.getHostDataPath())
      .withNetwork(await this.getNetwork())
      .withNetworkAliases('ord')
      .withBtcRpcUrl(`http://${bitcoinNetworkAlias}:18443`)
      .withBtcRpcUsername(this.bitcoinContainer.getRpcUser())
      .withBtcRpcPassword(this.bitcoinContainer.getRpcPass())
      .start()
  }

  unloadContainer() {
    this.bitcoinContainer?.stop()
    this.ordContainer?.stop()

    if (typeof this.roochContainer === 'number') {
      process.kill(this.roochContainer)
    } else {
      this.roochContainer?.stop()
    }

    this.tmpDir.removeCallback()
  }

  address(): RoochAddress {
    return this.keypair.getRoochAddress()
  }

  delay(second: number) {
    return new Promise((resolve) => setTimeout(resolve, second * 1000))
  }

  async signAndExecuteTransaction(tx: Transaction) {
    const result = await this.getClient().signAndExecuteTransaction({
      transaction: tx,
      signer: this.keypair,
    })

    return result.execution_info.status.type === 'executed'
  }

  private async getNetwork() {
    if (!this.network) {
      this.network = await new Network().start()
    }
    return this.network
  }

  shell(args: string[] | string): string {
    return execSync(`${typeof args === 'string' ? args : args.join(' ')}`, {
      encoding: 'utf-8',
    })
  }

  roochCommand(args: string[] | string): string {
    return execSync(`cargo run --bin rooch ${typeof args === 'string' ? args : args.join(' ')}`, {
      encoding: 'utf-8',
    })
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

  async cmdPublishPackage(
    packagePath: string,
    options: {
      namedAddresses: string
    } = {
      namedAddresses: 'rooch_examples=default',
    },
  ) {
    const result = this.roochCommand(
      `move publish -p ${packagePath} --named-addresses ${options.namedAddresses} --json`,
    )
    const { execution_info } = JSON.parse(result)

    return execution_info?.status?.type === 'executed'
  }

  /**
   * Retrieves the default account address.
   *
   * This method lists all accounts and returns the address of the first active account found.
   * If no active account is present, it throws an error.
   *
   * @returns {Promise<string>} A promise that resolves with the address of the default account.
   * @throws {Error} When no active account address is found.
   */
  async defaultCmdAddress(): Promise<string> {
    if (!_defaultCmdAddress) {
      const accounts = JSON.parse(this.roochCommand(['account', 'list', '--json']))

      if (Array.isArray(accounts)) {
        for (const account of accounts) {
          if (account.active) {
            _defaultCmdAddress = account.local_account.hex_address
          }
        }
      } else {
        const defaultAddr = accounts['default']
        _defaultCmdAddress = defaultAddr.hex_address
      }

      if (!_defaultCmdAddress) {
        throw new Error('No active account address')
      }
    }

    return _defaultCmdAddress
  }
}
