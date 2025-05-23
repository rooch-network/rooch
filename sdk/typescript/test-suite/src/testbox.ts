// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as fs from 'fs'
import * as net from 'net'
import path from 'node:path'
import { execSync } from 'child_process'
import { spawn } from 'child_process'
import debug from 'debug'
import tmp, { DirResult } from 'tmp'
import { Network, StartedNetwork } from 'testcontainers'

import { OrdContainer, StartedOrdContainer } from './container/ord.js'
import { RoochContainer, StartedRoochContainer } from './container/rooch.js'
import { BitcoinContainer, StartedBitcoinContainer } from './container/bitcoin.js'
import { PumbaContainer } from './container/pumba.js'
import { logConsumer } from './container/debug/log_consumer.js'

const log = debug('test-suite:testbox')

const ordNetworkAlias = 'ord'
const bitcoinNetworkAlias = 'bitcoind'

export class TestBox {
  tmpDir: DirResult
  network?: StartedNetwork
  ordContainer?: StartedOrdContainer
  bitcoinContainer?: StartedBitcoinContainer
  roochContainer?: StartedRoochContainer | number
  roochDir: string

  roochPort?: number
  private miningIntervalId: NodeJS.Timeout | null = null
  _defaultCmdAddress = ''

  constructor() {
    tmp.setGracefulCleanup()
    this.tmpDir = tmp.dirSync({ unsafeCleanup: true })
    this.roochDir = path.join(this.tmpDir.name, '.rooch_test')
    log('New TestBox rooch dir:', this.roochDir)
    fs.mkdirSync(this.roochDir, { recursive: true })
    this.roochCommand(['init', '--config-dir', `${this.roochDir}`, '--skip-password'])
    this.roochCommand(['env', 'switch', '--config-dir', `${this.roochDir}`, '--alias', 'local'])
  }

  async loadBitcoinEnv(customContainer?: BitcoinContainer, autoMining: boolean = false) {
    if (customContainer) {
      this.bitcoinContainer = await customContainer.start()
    } else {
      this.bitcoinContainer = await new BitcoinContainer()
        .withHostDataPath(this.tmpDir.name)
        .withNetwork(await this.getNetwork())
        .withNetworkAliases(bitcoinNetworkAlias)
        .start()
    }

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
      log('bitcoin container not init')
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
      if (port === 0) {
        port = await getUnusedPort()
      }

      // Generate a random port for metrics
      const metricsPort = await getUnusedPort()

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
            '--btc-sync-block-interval',
            '1',
          ],
        )
      }

      cmds.push('--traffic-per-second', '1')
      cmds.push('--traffic-burst-size', '5000')

      const result: string = await this.roochAsyncCommand(
        cmds,
        `JSON-RPC HTTP Server start listening 0.0.0.0:${port}`,
        [`METRICS_HOST_PORT=${metricsPort}`],
      )

      this.roochContainer = parseInt(result.toString().trim(), 10)
      this.roochPort = port

      return
    }

    this.roochCommand(['init', '--config-dir', `${this.roochDir}`, '--skip-password'])
    const container = new RoochContainer()

    // Find local Rooch binary path as in the 'local' mode
    const root = this.findRootDir('pnpm-workspace.yaml')
    const localRoochPath = path.join(root!, 'target', 'debug', 'rooch')

    // Verify local binary exists
    if (!fs.existsSync(localRoochPath)) {
      throw new Error(
        `Local Rooch binary not found at ${localRoochPath}. Make sure to build it first.`,
      )
    }

    // Configure container with local binary
    container.withLocalBinary(localRoochPath).withLogConsumer(logConsumer('rooch'))

    container
      .withNetwork(await this.getNetwork())
      .withDataDir('TMP')
      .withPort(6767)

    if (this.bitcoinContainer) {
      container
        .withBtcRpcUrl(`http://${bitcoinNetworkAlias}:18443`)
        .withBtcRpcUsername(this.bitcoinContainer.getRpcUser())
        .withBtcRpcPassword(this.bitcoinContainer.getRpcPass())
        .withBtcSyncBlockInterval(1) // Set sync interval to 1 second
    }

    this.roochContainer = await container.start()

    const rpcURL = this.roochContainer.getConnectionAddress()
    log('container rooch rpc:', rpcURL, 'roochDir:', this.roochDir)
    this.roochCommand([
      'env',
      'add',
      '--config-dir',
      `${this.roochDir}`,
      '--alias',
      'local',
      '--rpc',
      'http://' + rpcURL,
    ])
    this.roochCommand(['env', 'switch', '--config-dir', `${this.roochDir}`, '--alias', 'local'])
  }

  cleanEnv() {
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

  private buildRoochCommand(args: string[] | string, envs: string[] = []) {
    const root = this.findRootDir('pnpm-workspace.yaml')
    const roochDir = path.join(root!, 'target', 'debug')

    const envString = envs.length > 0 ? `${envs.join(' ')} ` : ''
    return `${envString} ${roochDir}/./rooch ${typeof args === 'string' ? args : args.join(' ')}`
  }

  // TODO: support container
  roochCommand(args: string[] | string, envs: string[] = []): string {
    return execSync(this.buildRoochCommand(args, envs), {
      encoding: 'utf-8',
    })
  }

  // TODO: support container
  async roochAsyncCommand(
    args: string[] | string,
    waitFor: string,
    envs: string[] = [],
  ): Promise<string> {
    return new Promise((resolve, reject) => {
      const command = this.buildRoochCommand(args, envs)
      const child = spawn(command, { shell: true })

      let output = ''
      let pidOutput = ''

      child.on('spawn', () => {
        if (child.pid) {
          pidOutput = child.pid.toString()
        } else {
          reject(new Error('Failed to obtain PID of the process'))
        }
      })

      child.stdout.on('data', (data) => {
        output += data.toString()

        if (output.includes(waitFor)) {
          resolve(pidOutput.trim())
        }
      })

      child.stderr.on('data', (data) => {
        process.stderr.write(data)
      })

      child.on('error', (error) => {
        reject(error)
      })

      child.on('close', () => {
        if (!output.includes(waitFor)) {
          reject(new Error('Expected output not found'))
        }
      })
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
    // let addr = await this.defaultCmdAddress()
    // let fixedNamedAddresses = options.namedAddresses.replace('default', addr)
    log('publish package:', packagePath, 'rooch Dir:', this.roochDir)

    const result = this.roochCommand(
      `move publish -p ${packagePath} --config-dir ${this.roochDir} --named-addresses ${options.namedAddresses} --json`,
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
    if (!this._defaultCmdAddress) {
      const accounts = JSON.parse(
        this.roochCommand(['account', 'list', '--config-dir', this.roochDir, '--json']),
      )

      if (Array.isArray(accounts)) {
        for (const account of accounts) {
          if (account.active) {
            this._defaultCmdAddress = account.local_account.hex_address
          }
        }
      } else {
        const defaultAddr = accounts['default']
        this._defaultCmdAddress = defaultAddr.hex_address
      }

      if (!this._defaultCmdAddress) {
        throw new Error('No active account address')
      }
    }

    return this._defaultCmdAddress
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

  private findRootDir(targetName: string) {
    let currentDir = process.cwd()

    while (currentDir !== path.parse(currentDir).root) {
      const targetPath = path.join(currentDir, targetName)
      if (fs.existsSync(targetPath)) {
        return currentDir
      }
      currentDir = path.dirname(currentDir)
    }

    return null
  }

  /**
   * Simulates network delay for the Rooch RPC endpoint.
   *
   * @param delayMs Delay in milliseconds to add to network requests
   * @param durationSec Duration of the simulated delay in seconds
   */
  async simulateRoochRpcDelay(delayMs: number, durationSec: number): Promise<void> {
    if (!(this.roochContainer instanceof StartedRoochContainer)) {
      throw new Error('This method only works with containerized Rooch instances')
    }

    const containerName = this.roochContainer.getContainerName()
    await PumbaContainer.simulateDelay(containerName, delayMs, durationSec)
  }

  /**
   * Simulates packet loss for the Rooch RPC endpoint.
   *
   * @param lossPercent Percentage of packets to drop (0-100)
   * @param durationSec Duration of the simulated loss in seconds
   */
  async simulateRoochRpcPacketLoss(lossPercent: number, durationSec: number): Promise<void> {
    if (!(this.roochContainer instanceof StartedRoochContainer)) {
      throw new Error('This method only works with containerized Rooch instances')
    }

    const containerName = this.roochContainer.getContainerName()
    await PumbaContainer.simulateLoss(containerName, lossPercent, durationSec)
  }

  /**
   * Simulates bandwidth limitation for the Rooch RPC endpoint.
   *
   * @param rate Bandwidth rate (e.g., "1mbit", "500kbit")
   * @param durationSec Duration of the bandwidth limitation in seconds
   */
  async simulateRoochRpcBandwidthLimit(rate: string, durationSec: number): Promise<void> {
    if (!(this.roochContainer instanceof StartedRoochContainer)) {
      throw new Error('This method only works with containerized Rooch instances')
    }

    const containerName = this.roochContainer.getContainerName()
    await PumbaContainer.simulateBandwidth(containerName, rate, durationSec)
  }
}

export async function getUnusedPort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer()
    server.on('error', (_err: any) => {
      server.close()
      getUnusedPort().then(resolve).catch(reject)
    })
    server.on('listening', () => {
      const address = server.address() as net.AddressInfo
      server.close()
      resolve(address.port)
    })
    server.listen(0)
  })
}
