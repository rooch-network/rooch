// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as fs from 'fs'
import * as net from 'net'
import path from 'node:path'
import { execSync, execFileSync, spawn } from 'child_process'
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
  metricsPort?: number
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
    serverArgs: string[] = [],
  ) {
    if (target && typeof target !== 'string') {
      await target.start()
      return
    }

    // The container test in the linux environment is incomplete, so use it first
    if (target === 'local') {
      // Use dynamic port allocation to avoid conflicts
      // For fixed ports (6767/6768), prefer dynamic allocation since these are commonly used
      if (port === 0 || port === 6767 || port === 6768) {
        port = await getUnusedPort()
        log(`Using dynamically allocated port ${port} for Rooch server`)
      } else {
        // For other specific ports, trust the caller's choice
        // If it fails, roochAsyncCommand will timeout with clear error
        log(`Using caller-specified port ${port} for Rooch server`)
      }

      // Generate a random port for metrics
      const metricsPort = await getUnusedPort()

      const cmds = ['server', 'start', '-n', 'local', '-d', 'TMP', '--port', port.toString()]
      if (serverArgs.length > 0) {
        cmds.push(...serverArgs)
      }

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
      this.metricsPort = metricsPort

      log(`Rooch server started with PID ${this.roochContainer} on port ${port}`)

      return
    }

    this.roochCommand(['init', '--config-dir', `${this.roochDir}`, '--skip-password'])
    const container = new RoochContainer()

    // Find local Rooch binary path as in the 'local' mode
    const root = this.findRootDir('pnpm-workspace.yaml')
    const buildProfile = process.env.ROOCH_BINARY_BUILD_PROFILE || 'debug'
    const localRoochPath = path.join(root!, 'target', buildProfile, 'rooch')

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
      const pid = this.roochContainer
      log(`Cleaning up Rooch server process with PID: ${pid}`)

      try {
        // Try graceful shutdown first with SIGTERM
        process.kill(pid, 'SIGTERM')
        log(`Sent SIGTERM to process ${pid}`)
      } catch (e: any) {
        // Process might already be dead
        log(`Failed to send SIGTERM to process ${pid}: ${e.message}`)
      }

      // Fallback: kill any process listening on the port (synchronous cleanup)
      if (this.roochPort) {
        try {
          log(`Cleaning up any process on port ${this.roochPort}`)
          // Use platform-appropriate command
          if (process.platform === 'win32') {
            // Windows: Use double %% for batch execution
            execSync(
              `for /f "tokens=5" %%a in ('netstat -aon ^| findstr :${this.roochPort}') do taskkill /F /PID %%a`,
              { stdio: 'ignore' },
            )
          } else {
            // Unix-like: Use lsof + kill
            execSync(`lsof -ti:${this.roochPort} | xargs kill -9 2>/dev/null || true`, {
              stdio: 'ignore',
            })
          }
          log(`Port ${this.roochPort} cleanup completed`)
        } catch (e) {
          // Ignore errors - port might already be free
          log(`Port cleanup completed (or was already free)`)
        }
      }
    } else {
      this.roochContainer?.stop()
    }

    this.tmpDir.removeCallback()
    log('Environment cleanup completed')
  }

  delay(second: number) {
    return new Promise((resolve) => setTimeout(resolve, second * 1000))
  }

  stop() {
    this.cleanEnv()
  }

  shell(args: string[] | string): string {
    return execSync(`${typeof args === 'string' ? args : args.join(' ')}`, {
      encoding: 'utf-8',
    })
  }

  private buildRoochCommand(args: string[] | string, envs: string[] = []) {
    const root = this.findRootDir('pnpm-workspace.yaml')
    // Use ROOCH_BINARY_BUILD_PROFILE environment variable or default to 'debug'
    const profile = process.env.ROOCH_BINARY_BUILD_PROFILE || 'debug'
    const roochDir = path.join(root!, 'target', profile)
    const roochBin = path.join(roochDir, 'rooch')

    // Parse environment variables from array format ["FOO=bar", "BAR=baz"]
    const extraEnv: { [key: string]: string } = {}
    for (const e of envs) {
      const [k, ...rest] = e.split('=')
      if (k && rest.length) {
        extraEnv[k] = rest.join('=')
      }
    }

    // Convert args to array format
    let roochArgs: string[] = typeof args === 'string' ? args.split(/\s+/) : args

    
    return {
      cmd: roochBin,
      args: roochArgs,
      env: Object.keys(extraEnv).length ? { ...process.env, ...extraEnv } : process.env,
    }
  }

  // TODO: support container
  roochCommand(args: string[] | string, envs: string[] = []): string {
    try {
      const { cmd, args: cmdArgs, env } = this.buildRoochCommand(args, envs)
      return execFileSync(cmd, cmdArgs, {
        encoding: 'utf-8',
        stdio: ['pipe', 'pipe', 'pipe'], // Capture stdout and stderr
        env,
      })
    } catch (error: any) {
      // Log the error for debugging
      log('roochCommand failed:', error.message)
      if (error.stdout) log('stdout:', error.stdout)
      if (error.stderr) log('stderr:', error.stderr)
      throw error
    }
  }

  // TODO: support container
  async roochAsyncCommand(
    args: string[] | string,
    waitFor: string,
    envs: string[] = [],
    timeoutMs: number = 30000, // 30 seconds default timeout
  ): Promise<string> {
    return new Promise((resolve, reject) => {
      const { cmd, args: cmdArgs, env } = this.buildRoochCommand(args, envs)
      const child = spawn(cmd, cmdArgs, { env })

      let output = ''
      let pidOutput = ''

      // Set up timeout
      const timeout = setTimeout(() => {
        child.kill('SIGTERM')
        // Force kill after 2 seconds if SIGTERM doesn't work
        setTimeout(() => {
          try {
            child.kill('SIGKILL')
          } catch (e) {
            // Process already dead, ignore
          }
        }, 2000)
        reject(
          new Error(`Timeout (${timeoutMs}ms) waiting for: ${waitFor}\nOutput so far: ${output}`),
        )
      }, timeoutMs)

      child.on('spawn', () => {
        if (child.pid) {
          pidOutput = child.pid.toString()
          log(`Spawned rooch process with PID: ${pidOutput}`)
        } else {
          clearTimeout(timeout)
          reject(new Error('Failed to obtain PID of the process'))
        }
      })

      child.stdout.on('data', (data) => {
        output += data.toString()
        log(`[rooch stdout]: ${data.toString().trim()}`)

        if (output.includes(waitFor)) {
          clearTimeout(timeout)
          log(`Found expected output: ${waitFor}`)
          resolve(pidOutput.trim())
        }
      })

      child.stderr.on('data', (data) => {
        const errStr = data.toString()
        log(`[rooch stderr]: ${errStr.trim()}`)
        process.stderr.write(data)
      })

      child.on('error', (error) => {
        clearTimeout(timeout)
        log(`Process error: ${error.message}`)
        reject(error)
      })

      child.on('close', (code) => {
        clearTimeout(timeout)
        if (!output.includes(waitFor)) {
          reject(
            new Error(
              `Process exited with code ${code}. Expected output not found: ${waitFor}\nFull output: ${output}`,
            ),
          )
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
    // The rooch CLI supports 'default' keyword in named addresses
    // Example: --named-addresses alice=0x1234,bob=default
    // 'default' will be automatically resolved to the active account address
    log(
      'publish package:',
      packagePath,
      'rooch Dir:',
      this.roochDir,
      'named addresses:',
      options.namedAddresses,
    )

    const result = this.roochCommand(
      `move publish -p ${packagePath} --config-dir ${this.roochDir} --named-addresses ${options.namedAddresses} --json`,
    )

    // The output contains both compilation logs and JSON result
    // Find the JSON object in the output (starts with '{' and ends with '}')
    const startIndex = result.indexOf('{')
    if (startIndex === -1) {
      log('Failed to find JSON in output:', result)
      return false
    }

    // Extract from first '{' to the end, it should be the JSON response
    const jsonPart = result.substring(startIndex)

    try {
      const { execution_info } = JSON.parse(jsonPart)
      return execution_info?.status?.type === 'executed'
    } catch (e) {
      log('Failed to parse JSON:', jsonPart.substring(0, 200), '...')
      log('Full result length:', result.length)
      return false
    }
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

  getMetricsPort(): number | undefined {
    return this.metricsPort
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

/**
 * Get an unused port by binding to port 0 (OS will assign a free port)
 * This ensures the port is available at the moment, though there's still
 * a small TOCTOU window. For critical applications, the server should
 * handle bind failures and retry with a new port.
 * @returns Promise that resolves to an available port number
 */
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
