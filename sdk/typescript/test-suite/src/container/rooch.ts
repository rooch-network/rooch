// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  AbstractStartedContainer,
  GenericContainer,
  StartedTestContainer,
  Wait,
} from 'testcontainers'

const ROOCH_PORT = 6767

export class RoochContainer extends GenericContainer {
  private networkName = 'local'
  private dataDir = 'TMP'
  private accountDir = '/root/.rooch'
  private port = ROOCH_PORT
  private ethRpcUrl?: string
  private btcRpcUrl?: string
  private btcRpcUsername?: string
  private btcRpcPassword?: string
  private btcEndBlockHeight?: number
  private btcSyncBlockInterval?: number
  private hostConfigPath?: string
  private trafficBurstSize?: number
  private trafficPerSecond?: number

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

  public withHostConfigPath(hostPath: string): this {
    this.hostConfigPath = hostPath
    return this
  }

  public withTrafficBurstSize(burstSize: number): this {
    this.trafficBurstSize = burstSize
    return this
  }

  public withTrafficPerSecond(perSecond: number): this {
    this.trafficPerSecond = perSecond
    return this
  }

  public async initializeRooch(): Promise<void> {
    if (!this.hostConfigPath) {
      throw new Error('Host config path not set. Call withHostConfigPath() before initializing.')
    }

    await new GenericContainer(this.imageName.string)
      .withStartupTimeout(10_000)
      .withBindMounts([{ source: this.hostConfigPath, target: this.accountDir }])
      .withCommand(['init', '--skip-password'])
      .start()

    await new GenericContainer(this.imageName.string)
      .withStartupTimeout(10_000)
      .withBindMounts([{ source: this.hostConfigPath, target: this.accountDir }])
      .withCommand(['env', 'switch', '--alias', 'local'])
      .start()
  }

  public async cleanRooch(): Promise<void> {
    if (!this.hostConfigPath) {
      throw new Error('Host config path not set. Call withHostConfigPath() before initializing.')
    }

    await new GenericContainer(this.imageName.string)
      .withStartupTimeout(10_000)
      .withBindMounts([{ source: this.hostConfigPath, target: this.accountDir }])
      .withCommand(['init', '--skip-password'])
      .start()

    await new GenericContainer(this.imageName.string)
      .withStartupTimeout(10_000)
      .withBindMounts([{ source: this.hostConfigPath, target: this.accountDir }])
      .withCommand(['env', 'switch', '--alias', 'local'])
      .start()
  }

  public override async start(): Promise<StartedRoochContainer> {
    if (!this.hostConfigPath) {
      throw new Error('Host config path not set. Call withHostConfigPath() before starting.')
    }

    this.withUser('root')
    this.withBindMounts([{ source: this.hostConfigPath, target: this.accountDir }])

    const command = [
      'server',
      'start',
      '-n',
      this.networkName,
      '-d',
      this.dataDir,
      '--port',
      this.port.toString(),
    ]

    if (this.ethRpcUrl) {
      command.push('--eth-rpc-url', this.ethRpcUrl)
    }

    if (this.btcRpcUrl) {
      command.push('--btc-rpc-url', this.btcRpcUrl)
    }

    if (this.btcRpcUsername) {
      command.push('--btc-rpc-username', this.btcRpcUsername)
    }

    if (this.btcRpcPassword) {
      command.push('--btc-rpc-password', this.btcRpcPassword)
    }

    if (this.btcEndBlockHeight !== undefined) {
      command.push('--btc-end-block-height', this.btcEndBlockHeight.toString())
    }

    if (this.btcSyncBlockInterval !== undefined) {
      command.push('--btc-sync-block-interval', this.btcSyncBlockInterval.toString())
    }

    if (this.trafficPerSecond !== undefined) {
      command.push('--traffic-per-second', this.trafficPerSecond.toString())
    }

    if (this.trafficBurstSize !== undefined) {
      command.push('--traffic-burst-size', this.trafficBurstSize.toString())
    }

    this.withCommand(command)

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
      this.trafficPerSecond,
    )
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
    private readonly trafficPerSecond?: number,
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
  public getTrafficPerSecond(): number | undefined {
    return this.trafficPerSecond
  }

  public getConnectionAddress(): string {
    return `${this.getHost()}:${this.getPort()}`
  }
}
