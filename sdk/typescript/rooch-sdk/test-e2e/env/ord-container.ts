// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { AbstractStartedContainer, GenericContainer, StartedTestContainer } from 'testcontainers'
import path from 'node:path'
import fs from 'fs'

export class OrdContainer extends GenericContainer {
  private btcRpcUrl = 'http:://bitcoind:18443'
  private btcRpcUsername = 'roochuser'
  private btcRpcPassword = 'roochpass'
  private hostDataPath?: string
  private bitcoinDataPath?: string

  constructor(image: string = 'bitseed/ord:0.18.0-burn') {
    super(image)
    this.withExposedPorts(80).withStartupTimeout(120_000)
  }

  public withBitcoinDataPath(bitcoinHostPath: string): this {
    this.bitcoinDataPath = bitcoinHostPath
    return this
  }
  public withHostDataPath(hostPath: string): this {
    this.hostDataPath = path.join(hostPath, 'ord')
    fs.mkdirSync(this.hostDataPath, { recursive: true })
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

  public override async start(): Promise<StartedOrdContainer> {
    if (!this.hostDataPath) {
      throw new Error('ord host data path not init.')
    }
    if (!this.bitcoinDataPath) {
      throw new Error('ord bitcoin host data path not init.')
    }

    this.withBindMounts([
      {
        source: this.bitcoinDataPath,
        target: '/root/.bitcoin',
      },
    ]).withBindMounts([
      {
        source: this.hostDataPath,
        target: '/data',
      },
    ])

    const command = ['--regtest']

    if (this.btcRpcUrl) {
      command.push(`--bitcoin-rpc-url=${this.btcRpcUrl}`)
    }

    if (this.btcRpcUsername) {
      command.push(`--bitcoin-rpc-username=${this.btcRpcUsername}`)
    }

    if (this.btcRpcPassword) {
      command.push(`--bitcoin-rpc-password=${this.btcRpcPassword}`)
    }

    command.push('server')

    this.withCommand(command)
    const container = await super.start()
    return new StartedOrdContainer(
      container,
      this.btcRpcUrl,
      this.btcRpcUsername,
      this.btcRpcPassword,
      this.hostDataPath,
    )
  }
}

export class StartedOrdContainer extends AbstractStartedContainer {
  constructor(
    startedTestContainer: StartedTestContainer,
    private btcRpcUrl: string,
    private btcRpcUsername: string,
    private btcRpcPassword: string,
    private hostDataPath: string,
  ) {
    super(startedTestContainer)
  }

  public getBtcRpcUrl(): string {
    return this.btcRpcUrl
  }

  public getBtcRpcUsername(): string {
    return this.btcRpcUsername
  }

  public getBtcRpcPassword(): string {
    return this.btcRpcPassword
  }

  public getHostDataPath(): string {
    return this.hostDataPath
  }

  execCmd(cmd: string[] | string) {
    const cmds = [
      'ord',
      '--regtest',
      '--bitcoin-rpc-url',
      this.getBtcRpcUrl(),
      '--bitcoin-rpc-username',
      this.getBtcRpcUsername(),
      '--bitcoin-rpc-password',
      this.getBtcRpcPassword(),
    ]

    cmds.push(...(typeof cmd === 'string' ? cmd.split(' ') : cmd))
    return this.exec(cmds)
  }
}
