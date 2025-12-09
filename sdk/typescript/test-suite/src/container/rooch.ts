// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  AbstractStartedContainer,
  GenericContainer,
  StartedTestContainer,
  Wait,
} from 'testcontainers'
import path from 'path'
import fs from 'fs'

const ROOCH_PORT = 6767

export class RoochContainer extends GenericContainer {
  private networkName = 'local'
  private dataDir = 'TMP'
  private port = ROOCH_PORT
  private ethRpcUrl?: string
  private btcRpcUrl?: string
  private btcRpcUsername?: string
  private btcRpcPassword?: string
  private btcEndBlockHeight?: number
  private btcSyncBlockInterval?: number
  private trafficBurstSize?: number
  private trafficReplenishIntervalS?: number
  private localBinaryPath?: string
  private skipInitialization = false

  constructor(image = 'ghcr.io/rooch-network/rooch:main_debug') {
    super(image)
    this.withExposedPorts(this.port)
      .withStartupTimeout(120_000)
      .withWaitStrategy(Wait.forLogMessage('JSON-RPC HTTP Server start listening'))
  }

  public withNetworkName(name: string): this {
    this.networkName = name
    return this
  }

  public withDataDir(dir: string): this {
    this.dataDir = dir
    return this
  }

  public withPort(port: number): this {
    this.port = port
    return this
  }

  public withEthRpcUrl(url: string): this {
    this.ethRpcUrl = url
    return this
  }

  public withBtcRpcUrl(url: string): this {
    this.btcRpcUrl = url
    return this
  }

  public withBtcRpcUsername(username: string): this {
    this.btcRpcUsername = username
    return this
  }

  public withBtcRpcPassword(password: string): this {
    this.btcRpcPassword = password
    return this
  }

  public withBtcEndBlockHeight(height: number): this {
    this.btcEndBlockHeight = height
    return this
  }

  public withBtcSyncBlockInterval(interval: number): this {
    this.btcSyncBlockInterval = interval
    return this
  }

  public withTrafficBurstSize(burstSize: number): this {
    this.trafficBurstSize = burstSize
    return this
  }

  public withTrafficReplenishIntervalS(intervalS: number): this {
    this.trafficReplenishIntervalS = intervalS
    return this
  }

  /**
   * DEPRECATED: Use withTrafficReplenishIntervalS instead for clarity.
   * Set the traffic per second (interval) for rate limiting.
   */
  public withTrafficPerSecond(perSecond: number): this {
    this.trafficReplenishIntervalS = perSecond
    return this
  }

  /**
   * Skip the initialization steps if you're using a pre-initialized config
   */
  public withSkipInitialization(skip: boolean = true): this {
    this.skipInitialization = skip
    return this
  }

  /**
   * Mount a locally compiled Rooch binary instead of using the binary in the container
   * @param localBinaryPath Absolute path to the local Rooch binary
   */
  public withLocalBinary(localBinaryPath: string): this {
    this.localBinaryPath = path.resolve(localBinaryPath)

    // Check if the binary exists
    if (!fs.existsSync(this.localBinaryPath)) {
      throw new Error(`Local Rooch binary not found at ${this.localBinaryPath}`)
    }

    return this
  }

  public override async start(): Promise<StartedRoochContainer> {
    this.withUser('root')

    // Add config bind mount
    const bindMounts = []

    // Add local binary bind mount if specified
    if (this.localBinaryPath) {
      bindMounts.push({ source: this.localBinaryPath, target: '/rooch/rooch' })
    }

    this.withBindMounts(bindMounts)

    // Create server start command
    const serverStartCmd = this.buildServerStartCommand()

    // Combine initialization and server start using bash
    let fullCommand: string

    if (this.skipInitialization) {
      fullCommand = serverStartCmd
    } else {
      fullCommand = `/rooch/rooch init --skip-password && \
        /rooch/rooch env switch --alias local && \
        ${serverStartCmd}`
    }

    this.withEntrypoint(['/bin/bash'])
    this.withCommand(['-c', fullCommand])

    const startedContainer = await super.start()

    return new StartedRoochContainer(
      startedContainer,
      this.networkName,
      this.dataDir,
      this.port,
      this.ethRpcUrl,
      this.btcRpcUrl,
      this.btcRpcUsername,
      this.btcRpcPassword,
      this.btcEndBlockHeight,
      this.btcSyncBlockInterval,
      this.trafficBurstSize,
      this.trafficReplenishIntervalS,
      this.localBinaryPath,
    )
  }

  /**
   * Build the server start command string with all options
   */
  private buildServerStartCommand(): string {
    let cmd = '/rooch/rooch server start'

    cmd += ` -n ${this.networkName}`
    cmd += ` -d ${this.dataDir}`
    cmd += ` --port ${this.port.toString()}`

    if (this.ethRpcUrl) {
      cmd += ` --eth-rpc-url ${this.ethRpcUrl}`
    }

    if (this.btcRpcUrl) {
      cmd += ` --btc-rpc-url ${this.btcRpcUrl}`
    }

    if (this.btcRpcUsername) {
      cmd += ` --btc-rpc-username ${this.btcRpcUsername}`
    }

    if (this.btcRpcPassword) {
      cmd += ` --btc-rpc-password ${this.btcRpcPassword}`
    }

    if (this.btcEndBlockHeight !== undefined) {
      cmd += ` --btc-end-block-height ${this.btcEndBlockHeight.toString()}`
    }

    if (this.btcSyncBlockInterval !== undefined) {
      cmd += ` --btc-sync-block-interval ${this.btcSyncBlockInterval.toString()}`
    }

    if (this.trafficReplenishIntervalS !== undefined) {
      cmd += ` --traffic-replenish-interval-s ${this.trafficReplenishIntervalS.toString()}`
    }

    if (this.trafficBurstSize !== undefined) {
      cmd += ` --traffic-burst-size ${this.trafficBurstSize.toString()}`
    }

    return cmd
  }
}

export class StartedRoochContainer extends AbstractStartedContainer {
  private readonly mappedPort: number

  constructor(
    startedTestContainer: StartedTestContainer,
    private readonly networkName: string,
    private readonly dataDir: string,
    private readonly containerPort: number,
    private readonly ethRpcUrl?: string,
    private readonly btcRpcUrl?: string,
    private readonly btcRpcUsername?: string,
    private readonly btcRpcPassword?: string,
    private readonly btcEndBlockHeight?: number,
    private readonly btcSyncBlockInterval?: number,
    private readonly trafficBurstSize?: number,
    private readonly trafficReplenishIntervalS?: number,
    private readonly localBinaryPath?: string,
  ) {
    super(startedTestContainer)
    this.mappedPort = startedTestContainer.getMappedPort(this.containerPort)
  }

  public getPort(): number {
    return this.mappedPort
  }

  public getNetworkName(): string {
    return this.networkName
  }

  public getDataDir(): string {
    return this.dataDir
  }

  public getEthRpcUrl(): string | undefined {
    return this.ethRpcUrl
  }

  public getBtcRpcUrl(): string | undefined {
    return this.btcRpcUrl
  }

  public getBtcRpcUsername(): string | undefined {
    return this.btcRpcUsername
  }

  public getBtcRpcPassword(): string | undefined {
    return this.btcRpcPassword
  }

  public getBtcEndBlockHeight(): number | undefined {
    return this.btcEndBlockHeight
  }

  public getBtcSyncBlockInterval(): number | undefined {
    return this.btcSyncBlockInterval
  }
  public getTrafficBurstSize(): number | undefined {
    return this.trafficBurstSize
  }
  public getTrafficReplenishIntervalS(): number | undefined {
    return this.trafficReplenishIntervalS
  }

  /**
   * DEPRECATED: Use getTrafficReplenishIntervalS instead for clarity.
   * Get the traffic per second (interval) for rate limiting.
   */
  public getTrafficPerSecond(): number | undefined {
    return this.trafficReplenishIntervalS
  }

  public getLocalBinaryPath(): string | undefined {
    return this.localBinaryPath
  }

  public getConnectionAddress(): string {
    return `${this.getHost()}:${this.getPort()}`
  }

  /**
   * Returns the Docker container name that can be used with Pumba for network simulations.
   * @returns The container name as a string
   */
  public getContainerName(): string {
    // The Docker container ID is available from the startedTestContainer
    // For Pumba, we need the full container name/ID
    return this.startedTestContainer.getName().slice(1) // Remove the leading slash
  }
}
