// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import * as os from 'os';
import tmp, { DirResult } from 'tmp'
import * as net from 'net';
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
  roochPort?: number
  private miningIntervalId: NodeJS.Timeout | null = null

  constructor() {
    this.tmpDir = this.createTmpDir()
  }

  async loadBitcoinEnv(customContainer?: BitcoinContainer, autoMining: boolean = false) {
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

    if (autoMining) {
      // Preprea Faucet
      await this.bitcoinContainer.prepareFaucet()

      // Start mining interval after Bitcoin container is started
      this.miningIntervalId = setInterval(async () => {
        if (this.bitcoinContainer) {
          await this.bitcoinContainer.mineBlock()
        }
      }, 1000) // Mine every 1 second
    }
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
      if (port == 0) {
        port = await getUnusedPort()
      }

      const cmds = ['server', 'start', '-n', 'local', '-d', 'TMP', '--port', port.toString()]

      if (this.bitcoinContainer) {
        cmds.push(
          ...[
            '--btc-rpc-url',
            this.bitcoinContainer.getRpcUrl(),
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
      this.roochPort = port;
      await this.delay(5)
      return
    }

    const container = new RoochContainer().withHostConfigPath(`${this.tmpDir.name}/.rooch`)
    await container.initializeRooch()

    container
      .withNetwork(await this.getNetwork())
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
    // Clear mining interval before stopping containers
    if (this.miningIntervalId) {
      clearInterval(this.miningIntervalId)
      this.miningIntervalId = null
    }

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

  getRoochServerAddress(): string | null {
    if (this.roochContainer && this.roochContainer instanceof StartedRoochContainer) {
      const startedRoochContainer = this.roochContainer as StartedRoochContainer
      return startedRoochContainer.getConnectionAddress()
    } else if (this.roochPort) {
      return `127.0.0.1:${this.roochPort}`
    }

    return `127.0.0.1:6767`
  }

  async getFaucetBTC(address: string, amount: number = 0.001): Promise<string> {
    if (!this.bitcoinContainer) {
      throw new Error('bitcoin container not start')
    }

    return await this.bitcoinContainer.getFaucetBTC(address, amount)
  }

  createTmpDir(): DirResult {
    tmp.setGracefulCleanup()
    const systemDir = getTempDirectory()
    return tmp.dirSync({ unsafeCleanup: true, tmpdir: systemDir, mode: 777 })
  }
}

export async function getUnusedPort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.on('error', (_err: any) => {
      server.close();
      getUnusedPort().then(resolve).catch(reject);
    });
    server.on('listening', () => {
      const address = server.address() as net.AddressInfo;
      server.close();
      resolve(address.port);
    });
    server.listen(0); 
  });
}

/**
 * Gets the temporary directory path.
 * Prioritizes $RUNNER_TEMP if available, otherwise uses the system's temp directory.
 * 
 * @returns {string} The path to the temporary directory
 */
function getTempDirectory(): string {
  // Check if RUNNER_TEMP environment variable is set (for GitHub Actions)
  const runnerTemp = process.env.RUNNER_TEMP;
  if (runnerTemp) {
      return runnerTemp;
  }

  // If RUNNER_TEMP is not set, use the system's temp directory
  return os.tmpdir();
}