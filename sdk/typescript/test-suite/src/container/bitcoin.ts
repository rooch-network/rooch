// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import fs from 'fs'
import path from 'node:path'
import * as crypto from 'crypto'
import {
  AbstractStartedContainer,
  GenericContainer,
  PortWithBinding,
  StartedTestContainer,
  Wait,
} from 'testcontainers'

const BITCOIN_PORTS = [18443, 18444, 28333, 28332]

export class BitcoinContainer extends GenericContainer {
  private rpcBind = '0.0.0.0:18443'
  private rpcUser = 'roochuser'
  private rpcPass = 'roochpass'
  private hostDataPath?: string

  constructor(image = 'lncm/bitcoind:v25.1') {
    super(image)
    const s: PortWithBinding[] = BITCOIN_PORTS.map((item) => {
      return {
        host: 0,
        container: item,
      }
    })
    this.withExposedPorts(...s).withStartupTimeout(120_000)
  }

  public withHostDataPath(hostPath: string): this {
    this.hostDataPath = path.join(hostPath, 'bitcoin')
    fs.mkdirSync(this.hostDataPath, { recursive: true })
    return this
  }

  public withRpcBind(rpcBind: string): this {
    this.rpcBind = rpcBind
    return this
  }

  public withRpcUser(rpcUser: string): this {
    this.rpcUser = rpcUser
    return this
  }

  public withRpcPass(rpcPass: string): this {
    this.rpcPass = rpcPass
    return this
  }

  private generateRpcauth(): string {
    const salt = crypto.randomBytes(16).toString('hex')
    const hmac = crypto.createHmac('sha256', salt)
    hmac.update(this.rpcPass)
    const passwordHmac = hmac.digest('hex')

    return `${this.rpcUser}:${salt}$${passwordHmac}`
  }

  public override async start(): Promise<StartedBitcoinContainer> {
    if (!this.hostDataPath) {
      throw new Error(
        'Bitcoin host config path not set. Call withHostDataPath() before initializing.',
      )
    }

    const rpcauth = this.generateRpcauth()

    this.withEnvironment({
      RPC_BIND: this.rpcBind,
      RPC_USER: this.rpcUser,
      RPC_PASS: this.rpcPass,
      RPC_AUTH: rpcauth,
    })
      .withWaitStrategy(Wait.forLogMessage('txindex thread start'))
      .withStartupTimeout(120000)
      .withBindMounts([
        {
          source: this.hostDataPath!,
          target: '/data/.bitcoin',
        },
      ])

    this.withCommand([
      '-chain=regtest',
      '-txindex=1',
      '-fallbackfee=0.00001',
      '-zmqpubrawblock=tcp://0.0.0.0:28332',
      '-zmqpubrawtx=tcp://0.0.0.0:28333',
      '-rpcallowip=0.0.0.0/0',
      `-rpcbind=${this.rpcBind}`,
      `-rpcauth=${rpcauth}`,
    ])

    const container = await super.start()
    return new StartedBitcoinContainer(
      container,
      this.rpcBind,
      this.rpcUser,
      this.rpcPass,
      this.hostDataPath,
    )
  }
}

export class StartedBitcoinContainer extends AbstractStartedContainer {
  private readonly ports: { [key: number]: number }
  private preminedAddress: string | null = null

  constructor(
    startedTestContainer: StartedTestContainer,
    private readonly rpcBind: string,
    private readonly rpcUser: string,
    private readonly rpcPass: string,
    private readonly hostDataPath: string,
  ) {
    super(startedTestContainer)
    this.ports = BITCOIN_PORTS.reduce(
      (acc, port) => {
        acc[port] = startedTestContainer.getMappedPort(port)
        return acc
      },
      {} as { [key: number]: number },
    )
  }

  public getPort(port: number): number {
    return this.ports[port]
  }

  public getRpcBind(): string {
    return this.rpcBind
  }

  public getRpcUser(): string {
    return this.rpcUser
  }

  public getRpcPass(): string {
    return this.rpcPass
  }

  public getHostDataPath(): string {
    return this.hostDataPath
  }

  public getRpcUrl(): string {
    return `http://${this.getHost()}:${this.getPort(18443)}`
  }

  public async executeRpcCommand(command: string, params: any[] = []): Promise<any> {
    return this.executeRpcCommandRaw(
      [],
      command,
      params.map((param) => JSON.stringify(param)),
    )
  }

  public async executeRpcCommandRaw(
    opts: string[],
    command: string,
    params: string[] = [],
  ): Promise<any> {
    const cmd = ['bitcoin-cli', '-regtest', ...opts, command, ...params]
    const result = await this.startedTestContainer.exec(cmd)

    if (result.exitCode !== 0) {
      throw new Error(
        `executeRpcCommand failed with exit code ${result.exitCode} for command: ${command}`,
      )
    }

    return result.output
  }

  public async prepareFaucet() {
    await this.executeRpcCommand('createwallet', ['faucet_wallet'])

    const getnewaddressOutput = await this.executeRpcCommandRaw(
      [`-rpcwallet="faucet_wallet"`],
      'getnewaddress',
      [],
    )
    this.preminedAddress = getnewaddressOutput.trim()

    if (this.preminedAddress) {
      await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], 'generatetoaddress', [
        '101',
        this.preminedAddress,
      ])
    }
  }

  public async getFaucetBTC(address: string, amount: number = 0.001): Promise<string> {
    if (!this.preminedAddress) {
      throw new Error('Failed to generate pre-mined address')
    }

    const txid = await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], 'sendtoaddress', [
      address,
      String(amount),
    ])
    await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], 'generatetoaddress', [
      '1',
      this.preminedAddress,
    ])

    return txid
  }

  public async mineBlock(): Promise<void> {
    if (!this.preminedAddress) {
      throw new Error('Failed to generate pre-mined address')
    }

    await this.executeRpcCommandRaw([`-rpcwallet="faucet_wallet"`], 'generatetoaddress', [
      '1',
      await this.preminedAddress,
    ])
  }
}