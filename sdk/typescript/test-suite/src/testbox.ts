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

  // Whether to preserve the temporary directory (including data dir and logs)
  private keepTmp: boolean

  roochPort?: number
  metricsPort?: number
  private miningIntervalId: NodeJS.Timeout | null = null
  _defaultCmdAddress = ''

  constructor() {
    const baseDir = process.env.TESTBOX_BASE_DIR
    const keepTmp =
      process.env.TESTBOX_KEEP_TMP === '1' || process.env.TESTBOX_KEEP_TMP?.toLowerCase() === 'true'

    // Cache the flag so cleanup logic can respect it later
    this.keepTmp = keepTmp

    console.error('🔧 TestBox constructor: TESTBOX_KEEP_TMP =', process.env.TESTBOX_KEEP_TMP)
    console.error('🔧 TestBox constructor: keepTmp =', keepTmp)

    let tmpDir: DirResult
    if (baseDir) {
      fs.mkdirSync(baseDir, { recursive: true })
      const name = fs.mkdtempSync(path.join(baseDir, 'testbox-'))
      tmpDir = {
        name,
        removeCallback: () => {
          if (!keepTmp) {
            fs.rmSync(name, { recursive: true, force: true })
          } else {
            console.error('🔧 Skipping temp directory removal (keepTmp=true)')
          }
        },
      } as DirResult
    } else {
      tmp.setGracefulCleanup()
      tmpDir = tmp.dirSync({ unsafeCleanup: !keepTmp, keep: keepTmp })
      console.error('🔧 Using tmp.dirSync with unsafeCleanup:', !keepTmp, 'keep:', keepTmp)
    }

    this.tmpDir = tmpDir
    this.roochDir = path.join(this.tmpDir.name, '.rooch_test')
    console.error('🔧 New TestBox rooch dir:', this.roochDir)
    fs.mkdirSync(this.roochDir, { recursive: true })

    this.initRoochConfig()

    console.error('🔧 TestBox constructor completed')
  }

  private initRoochConfig() {
    console.error('🔧 Running rooch init with config-dir:', this.roochDir)
    try {
      const initResult = this.roochCommand([
        'init',
        '--config-dir',
        this.roochDir,
        '--skip-password',
      ])
      console.error('🔧 rooch init result:', initResult.substring(0, 200))
    } catch (error: any) {
      console.error('🔧 rooch init failed:', error.message)
      throw error
    }

    // Verify rooch.yaml was created
    const configPath = path.join(this.roochDir, 'rooch.yaml')
    if (fs.existsSync(configPath)) {
      console.error('🔧 rooch.yaml created successfully at:', configPath)
    } else {
      console.error('🔧 WARNING: rooch.yaml not found at:', configPath)
    }

    console.error('🔧 Running rooch env switch with config-dir:', this.roochDir)
    this.roochCommand(['env', 'switch', '--config-dir', this.roochDir, '--alias', 'local'])
  }

  private ensureLocalEnv(rpcUrl: string) {
    console.error('🔧 Updating rooch CLI env to server:', rpcUrl)
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
    log(`Rooch CLI env "local" switched to ${rpcUrl}`)
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
      console.error('🔄 Entered local target branch, port param:', port)
      // Use dynamic port allocation to avoid conflicts
      if (port === 0 || port === undefined) {
        console.error('🔄 Getting unused port...')
        port = await getUnusedPort()
        console.error(`🔄 Using dynamically allocated port ${port} for Rooch server`)
      } else {
        // For other specific ports, trust the caller's choice
        // If it fails, roochAsyncCommand will timeout with clear error
        console.error(`🔄 Using caller-specified port ${port} for Rooch server`)
      }

      // Generate a random port for metrics
      console.error('🔄 Getting metrics port...')
      const metricsPort = await getUnusedPort()
      console.error(`🔄 Metrics port: ${metricsPort}`)

      const dataDir = path.join(this.roochDir, 'data')
      fs.mkdirSync(dataDir, { recursive: true })
      const cmds = [
        'server',
        'start',
        '--config-dir',
        this.roochDir,
        '-n',
        'local',
        '-d',
        dataDir,
        '--port',
        port.toString(),
      ]
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

      // Only add default rate limit config if not already specified in serverArgs
      // Note: traffic-per-second is the interval (in seconds) to replenish one quota element
      // e.g., 0.1 means replenish 1 quota every 0.1s = 10 requests/second
      const hasTrafficPerSecond = serverArgs.includes('--traffic-per-second')
      const hasTrafficBurstSize = serverArgs.includes('--traffic-burst-size')

      if (!hasTrafficPerSecond) {
        cmds.push('--traffic-per-second', '0.01')
      }
      if (!hasTrafficBurstSize) {
        cmds.push('--traffic-burst-size', '5000')
      }

      console.error('🚀 About to call roochAsyncCommand with cmds:', JSON.stringify(cmds, null, 2))
      console.error(
        '🚀 Waiting for output:',
        `JSON-RPC HTTP Server start listening 0.0.0.0:${port}`,
      )
      console.error('🚀 Using ROOCH_CONFIG_DIR:', this.roochDir)
      const result: string = await this.roochAsyncCommand(
        cmds,
        `JSON-RPC HTTP Server start listening 0.0.0.0:${port}`,
        [`METRICS_HOST_PORT=${metricsPort}`, `ROOCH_CONFIG_DIR=${this.roochDir}`],
      )
      console.error('✅ roochAsyncCommand returned:', result)

      this.roochContainer = parseInt(result.toString().trim(), 10)
      this.roochPort = port
      this.metricsPort = metricsPort

      // Ensure the CLI env points to the dynamically started local server
      const rpcUrl = `http://127.0.0.1:${port}`
      console.error('🔧 Updating rooch CLI env to dynamic local server:', rpcUrl)
      this.ensureLocalEnv(rpcUrl)

      log(`Rooch CLI env "local" switched to ${rpcUrl}`)

      log(`Rooch server started with PID ${this.roochContainer} on port ${port}`)

      return
    }

    this.roochCommand(['init', '--config-dir', this.roochDir, '--skip-password'])
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
    this.roochCommand(['env', 'switch', '--config-dir', this.roochDir, '--alias', 'local'])
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

      // Wait for process to exit gracefully (up to 5 seconds)
      const startTime = Date.now()
      const maxWaitMs = 5000
      while (Date.now() - startTime < maxWaitMs) {
        try {
          // Check if process is still alive (signal 0 doesn't kill, just checks)
          process.kill(pid, 0)
          // Process still alive, wait a bit
          execSync('sleep 0.5', { stdio: 'ignore' })
        } catch (e) {
          // Process is dead, we can proceed
          log(`Process ${pid} has exited`)
          break
        }
      }

      // Fallback: force kill any process listening on the port
      if (this.roochPort) {
        try {
          log(`Force killing any process on port ${this.roochPort}`)
          // Use platform-appropriate command
          if (process.platform === 'win32') {
            // Windows: Use double %% for batch execution
            execSync(
              `for /f "tokens=5" %%a in ('netstat -aon ^| findstr :${this.roochPort}') do taskkill /F /PID %%a`,
              { stdio: 'ignore' },
            )
          } else {
            // Unix-like: Use lsof + kill -9 (SIGKILL)
            execSync(`lsof -ti:${this.roochPort} | xargs kill -9 2>/dev/null || true`, {
              stdio: 'ignore',
            })
          }
          log(`Port ${this.roochPort} cleanup completed`)
        } catch (e) {
          // Ignore errors - port might already be free
          log(`Port cleanup completed (or was already free)`)
        }

        // Wait a bit more after force kill for file handles to be released
        try {
          execSync('sleep 1', { stdio: 'ignore' })
        } catch (e) {
          // Ignore
        }
      }

      // Reset roochContainer reference after stopping the process
      this.roochContainer = undefined
    } else {
      this.roochContainer?.stop()
      this.roochContainer = undefined
    }

    // Try to remove temp directory, but don't fail if it can't be removed
    // Respect keepTmp: in debugging/investigation flows we must preserve data dir
    if (this.keepTmp) {
      log('Temp directory preservation enabled (keepTmp=true); skipping cleanup')
    } else {
      try {
        this.tmpDir.removeCallback()
        log('Temp directory cleanup completed')
      } catch (e: any) {
        log(`Warning: Failed to remove temp directory: ${e.message}`)
        // Try force removal as fallback
        try {
          execSync(`rm -rf "${this.tmpDir.name}" 2>/dev/null || true`, { stdio: 'ignore' })
          log('Temp directory force removed')
        } catch (e2) {
          log('Warning: Force removal also failed, temp directory may remain')
        }
      }
    }
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
    const roochArgs: string[] = typeof args === 'string' ? args.split(/\s+/) : args

    const env = { ...process.env, ROOCH_CONFIG_DIR: this.roochDir, ...extraEnv }

    return {
      cmd: roochBin,
      args: roochArgs,
      env,
    }
  }

  // TODO: support container
  roochCommand(args: string[] | string, envs: string[] = []): string {
    try {
      const { cmd, args: cmdArgs, env } = this.buildRoochCommand(args, envs)
      console.info('🔍 Executing command:', cmd, cmdArgs.join(' '))
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

  // Check TCP endpoint connectivity with retry logic
  async waitForTcpEndpoint(port: number, maxAttempts: number = 60): Promise<void> {
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        await new Promise<void>((resolve, reject) => {
          const net = require('net')
          const socket = new net.Socket()

          socket.setTimeout(2000, () => {
            socket.destroy()
            reject(new Error('Connection timeout'))
          })

          socket.connect(port, 'localhost', () => {
            socket.destroy()
            resolve()
          })

          socket.on('error', (error: Error) => {
            socket.destroy()
            reject(error)
          })
        })
        log(`✅ TCP port ${port} is ready after ${attempt} attempt(s)`)
        return
      } catch (error: any) {
        if (attempt === maxAttempts) {
          throw new Error(
            `TCP port ${port} not ready after ${maxAttempts} attempts: ${error.message}`,
          )
        }
        log(`⏳ TCP port ${port} not ready (attempt ${attempt}/${maxAttempts}): ${error.message}`)
        await new Promise((resolve) => setTimeout(resolve, 2000))
      }
    }
  }

  // Check HTTP endpoint availability with retry logic
  async waitForHttpEndpoint(url: string, maxAttempts: number = 30): Promise<void> {
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        const response = await fetch(url, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: 'rooch_getChainID',
            params: [],
          }),
          signal: AbortSignal.timeout(5000),
        })

        if (response.ok) {
          const data = await response.json()
          if (data.result) {
            log(`✅ HTTP endpoint ${url} is ready after ${attempt} attempt(s)`)
            return
          }
        }
        throw new Error(`Invalid response: ${response.status}`)
      } catch (error: any) {
        if (attempt === maxAttempts) {
          throw new Error(
            `HTTP endpoint ${url} not ready after ${maxAttempts} attempts: ${error.message}`,
          )
        }
        log(
          `⏳ HTTP endpoint ${url} not ready (attempt ${attempt}/${maxAttempts}): ${error.message}`,
        )
        await new Promise((resolve) => setTimeout(resolve, 3000))
      }
    }
  }

  // Perform both TCP and HTTP health checks for server readiness
  async performHealthChecks(port: number): Promise<void> {
    try {
      // First check TCP connectivity
      await this.waitForTcpEndpoint(port)

      // Then check HTTP endpoint
      const httpUrl = `http://localhost:${port}`
      await this.waitForHttpEndpoint(httpUrl)

      log(`🎉 Server health checks passed - port ${port} is fully ready`)
    } catch (error: any) {
      throw new Error(`Health checks failed for port ${port}: ${error.message}`)
    }
  }

  // TODO: support container
  async roochAsyncCommand(
    args: string[] | string,
    waitFor: string,
    envs: string[] = [],
    timeoutMs: number = 300000, // 5 minutes default timeout for server startup
  ): Promise<string> {
    return new Promise((resolve, reject) => {
      const maxBufferedOutput = 20_000 // keep only the tail to avoid unbounded memory growth
      const { cmd, args: cmdArgs, env } = this.buildRoochCommand(args, envs)
      console.log('🔍 Executing command:', cmd, cmdArgs.join(' '))
      console.log('🔍 Full command string:', `${cmd} ${cmdArgs.join(' ')}`)
      console.log(`🔍 Using timeout: ${timeoutMs}ms for waitFor: ${waitFor}`)
      const logDir = process.env.TESTBOX_LOG_DIR || this.roochDir
      fs.mkdirSync(logDir, { recursive: true })
      const stdoutPath = path.join(logDir, 'rooch-server.stdout.log')
      const stderrPath = path.join(logDir, 'rooch-server.stderr.log')
      console.log('📝 Rooch server logs at:', stdoutPath, stderrPath)

      const child = spawn(cmd, cmdArgs, { env })
      const stdoutStream = fs.createWriteStream(stdoutPath, { flags: 'a' })
      const stderrStream = fs.createWriteStream(stderrPath, { flags: 'a' })

      let output = ''
      let pidOutput = ''
      let settled = false

      const stopBuffering = () => {
        settled = true
        output = output.slice(-maxBufferedOutput)
      }

      const cleanup = () => {
        clearTimeout(timeout)
      }

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
        }, 10000)
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

      const handleData = (data: Buffer) => {
        if (!settled) {
          output = (output + data.toString()).slice(-maxBufferedOutput)
        }

        if (!settled && output.includes(waitFor)) {
          log(`Found expected output: ${waitFor}`)
          stopBuffering()

          // Add health checks if this looks like a server startup
          if (waitFor.includes('JSON-RPC HTTP Server start listening')) {
            // Extract port from the startup message
            const portMatch = output.match(/JSON-RPC HTTP Server start listening 0\.0\.0\.0:(\d+)/)
            const port = portMatch ? parseInt(portMatch[1]) : 50051 // Default port

            log(`🔍 Starting health checks for port ${port}...`)

            // Run health checks in background
            this.performHealthChecks(port)
              .then(() => {
                log(`✅ Server is ready for use`)
                cleanup()
                resolve(pidOutput.trim())
              })
              .catch((error) => {
                cleanup()
                reject(new Error(`Health checks failed: ${error.message}`))
              })
          } else {
            // For non-server commands, resolve immediately
            cleanup()
            resolve(pidOutput.trim())
          }
        }
      }

      child.stdout.on('data', (data) => {
        stdoutStream.write(data)
        log(`[rooch stdout]: ${data.toString().trim()}`)
        handleData(data)
      })

      child.stderr.on('data', (data) => {
        const errStr = data.toString()
        stderrStream.write(data)
        log(`[rooch stderr]: ${errStr.trim()}`)
        process.stderr.write(data)
        handleData(data)
      })

      child.on('error', (error) => {
        cleanup()
        log(`Process error: ${error.message}`)
        reject(error)
      })

      child.on('close', (code) => {
        cleanup()
        stdoutStream.end()
        stderrStream.end()
        if (!settled && !output.includes(waitFor)) {
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

    const result = this.roochCommand([
      'move',
      'publish',
      '-p',
      packagePath,
      '--config-dir',
      this.roochDir,
      '--named-addresses',
      options.namedAddresses,
      '--skip-client-compat-check',
      '--json',
    ])

    console.log('publish result:', result.substring(0, 400), '...')

    // The output contains both compilation logs and JSON result
    // Find the JSON object in the output (starts with '{' and ends with '}')
    const startIndex = result.indexOf('{')
    if (startIndex === -1) {
      console.log('Failed to find JSON in output:', result)
      return false
    }

    // Extract from first '{' to the end, it should be the JSON response
    const jsonPart = result.substring(startIndex)

    try {
      const { execution_info } = JSON.parse(jsonPart)
      console.log('execution_info:', JSON.stringify(execution_info, null, 2))
      const isExecuted = execution_info?.status?.type === 'executed'
      console.log('is executed:', isExecuted)
      return isExecuted
    } catch (e) {
      console.log('Failed to parse JSON:', jsonPart.substring(0, 200), '...')
      console.log('Full result length:', result.length)
      console.log('Error:', e)
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
export async function getUnusedPort(maxAttempts = 3): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer()

    server.on('error', (err: any) => {
      server.close()

      // Surface sandbox restrictions immediately instead of recursing forever
      if (err?.code === 'EPERM') {
        reject(
          new Error(
            'Port binding not permitted (EPERM). The test harness likely blocks opening local ports; rerun with network permissions or disable the sandbox.',
          ),
        )
        return
      }

      if (maxAttempts <= 1) {
        reject(new Error(`Failed to allocate unused port: ${err?.message || err}`))
        return
      }

      getUnusedPort(maxAttempts - 1).then(resolve).catch(reject)
    })

    server.on('listening', () => {
      const address = server.address() as net.AddressInfo
      server.close()
      resolve(address.port)
    })

    try {
      server.listen(0)
    } catch (err: any) {
      server.close()
      reject(new Error(`Failed to initiate port listener: ${err?.message || err}`))
    }
  })
}
