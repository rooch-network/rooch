// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import tmp, { DirResult } from 'tmp'
import { execSync } from 'child_process'
import { Network, StartedNetwork } from 'testcontainers'

import { OrdContainer, StartedOrdContainer } from './container/ord.js'
import { RoochContainer, StartedRoochContainer } from './container/rooch.js'
import { BitcoinContainer, StartedBitcoinContainer } from './container/bitcoin.js'

const ordNetworkAlias = 'ord'
const bitcoinNetworkAlias = 'bitcoind'
let _defaultCmdAddress = ''

export class TestBox {
  tmpDir: DirResult
  network?: StartedNetwork
  ordContainer?: StartedOrdContainer
  bitcoinContainer?: StartedBitcoinContainer
  roochContainer?: StartedRoochContainer | number

  constructor() {
    tmp.setGracefulCleanup()
    this.tmpDir = tmp.dirSync({ unsafeCleanup: true })
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

    await this.delay(5)
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
      .withNetworkAliases(ordNetworkAlias)
      .withBtcRpcUrl(`http://${bitcoinNetworkAlias}:18443`)
      .withBtcRpcUsername(this.bitcoinContainer.getRpcUser())
      .withBtcRpcPassword(this.bitcoinContainer.getRpcPass())
      .start()

    await this.delay(5)
  }

  async loadRoochEnv(
    target: RoochContainer | 'local' | 'container' = 'local',
    port: number = 6767,
  ) {
    if (target && typeof target !== 'string') {
      await target.start()
      return
    }

    // The container test in the linux environment is incomplete, so use it first
    if (target === 'local') {
      const cmds = ['server', 'start', '-n', 'local', '-d', 'TMP', '--port', port.toString()]

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

      await this.delay(5)
      return
    }

    const container = new RoochContainer().withHostConfigPath(`${this.tmpDir.name}/.rooch`)
    await container.initializeRooch()

    container
      .withNetwork(await this.getNetwork())
      .withNetworkName('local')
      .withDataDir('TMP')
      .withPort(port)

    if (this.bitcoinContainer) {
      container
        .withBtcRpcUrl(`http://${bitcoinNetworkAlias}:18443`)
        .withBtcRpcUsername(this.bitcoinContainer.getRpcUser())
        .withBtcRpcPassword(this.bitcoinContainer.getRpcPass())
        .withBtcSyncBlockInterval(1) // Set sync interval to 1 second
    }

    this.roochContainer = await container.start()
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

  delay(second: number) {
    return new Promise((resolve) => setTimeout(resolve, second * 1000))
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

  private async getNetwork() {
    if (!this.network) {
      this.network = await new Network().start()
    }
    return this.network
  }
}
