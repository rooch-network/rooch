// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Args } from '../bcs/index.js'
import { RoochClient } from '../client/index.js'
import { Ed25519Keypair } from '../keypairs/index.js'
import { Transaction } from '../transactions/index.js'
import { BitcoinAddress, RoochAddress } from '../address/index.js'
import { Authenticator, PublicKey, SignatureScheme, Signer } from '../crypto/index.js'

import { CreateSessionArgs } from './types.js'
import { fromHEX } from '../utils/index.js'
import { Bytes } from '../types/index.js'

const DEFAULT_MAX_INACTIVE_INTERVAL = 1200 // second
const REQUIRED_SCOPE = '0x3::session_key::remove_session_key_entry'

type InnerCreateSessionArgs = {
  client: RoochClient
  signer: Signer
} & CreateSessionArgs

type InnerBuildSessionArgs = {
  addr: BitcoinAddress
} & CreateSessionArgs

export class Session extends Signer {
  protected readonly appName: string
  protected readonly appUrl: string
  protected readonly scopes: string[]
  protected readonly keypair: Ed25519Keypair
  protected readonly maxInactiveInterval: number
  protected readonly bitcoinAddress: BitcoinAddress
  protected readonly roochAddress: RoochAddress
  protected lastActiveTime: number

  protected constructor(
    appName: string,
    appUrl: string,
    scopes: string[],
    roochAddress: RoochAddress,
    bitcoinAddress: BitcoinAddress,
    keypair?: Ed25519Keypair,
    maxInactiveInterval?: number,
    localCreateSessionTime?: number,
    lastActiveTime?: number,
  ) {
    super()
    this.appName = appName
    this.appUrl = appUrl
    this.scopes = scopes
    this.roochAddress = roochAddress
    this.bitcoinAddress = bitcoinAddress
    this.keypair = keypair ?? Ed25519Keypair.generate()
    this.maxInactiveInterval = maxInactiveInterval ?? DEFAULT_MAX_INACTIVE_INTERVAL
    this.localCreateSessionTime = localCreateSessionTime ?? Date.now()
    this.lastActiveTime = lastActiveTime || this.localCreateSessionTime
  }

  protected readonly localCreateSessionTime: number

  public static async CREATE(input: InnerCreateSessionArgs): Promise<Session> {
    return this.formatArgs(input, input.signer.getBitcoinAddress()).build(
      input.client,
      input.signer,
    )
  }

  public static Build(input: InnerBuildSessionArgs): string {
    return this.formatArgs(input, input.addr).toJSON()
  }
  static formatArgs(input: CreateSessionArgs, addr: BitcoinAddress): Session {
    const parsedScopes = input.scopes.map((scope) => {
      if (typeof scope !== 'string') {
        return `${scope.address}::${scope.module}::${scope.function}`
      }
      if (scope.split('::').length !== 3) throw Error('invalid scope')
      return scope
    })

    const allOx3 = '0x3::*::*'

    if (!parsedScopes.find((item) => item === allOx3 || item === REQUIRED_SCOPE)) {
      parsedScopes.push(REQUIRED_SCOPE)
    }

    return new Session(
      input.appName,
      input.appUrl,
      parsedScopes,
      addr.genRoochAddress(),
      addr,
      input.keypair,
      input.maxInactiveInterval,
    )
  }

  static fromJson(jsonObj: any) {
    const {
      appName,
      appUrl,
      scopes,
      secretKey,
      maxInactiveInterval,
      bitcoinAddress,
      roochAddress,
      localCreateSessionTime,
      lastActiveTime,
    } = jsonObj

    return new Session(
      appName,
      appUrl,
      scopes,
      new RoochAddress(roochAddress),
      new BitcoinAddress(bitcoinAddress),
      Ed25519Keypair.fromSecretKey(secretKey),
      maxInactiveInterval,
      localCreateSessionTime,
      lastActiveTime,
    )
  }

  sign(input: Bytes): Promise<Bytes> {
    // if (this.lastActiveTime + this.maxInactiveInterval < Date.now() / 1000) {
    //   throw Error('Session is Expired')
    // }
    this.lastActiveTime = Date.now()
    return this.keypair.sign(input)
  }

  signTransaction(input: Transaction): Promise<Authenticator> {
    return Authenticator.rooch(input.hashData(), this)
  }

  getRoochAddress(): RoochAddress {
    return this.roochAddress
  }

  getBitcoinAddress(): BitcoinAddress {
    return this.bitcoinAddress
  }

  getKeyScheme(): SignatureScheme {
    return this.keypair.getKeyScheme()
  }

  getPublicKey(): PublicKey<RoochAddress> {
    return this.keypair.getPublicKey()
  }

  getCreateTime(): number {
    return this.localCreateSessionTime
  }

  getAuthKey(): string {
    return this.keypair.getRoochAddress().toHexAddress()
  }

  protected async build(client: RoochClient, signer: Signer) {
    const [addrs, mods, fns] = this.scopes
      .map((scope) => {
        return scope.split('::')
      })
      .reduce(
        (acc: Array<Array<string>>, val: Array<string>) => {
          acc[0].push(val[0])
          acc[1].push(val[1])
          acc[2].push(val[2])
          return acc
        },
        [[], [], []],
      )

    const tx = new Transaction()
    tx.callFunction({
      target: '0x3::session_key::create_session_key_with_multi_scope_entry',
      args: [
        Args.string(this.appName),
        Args.string(this.appUrl),
        Args.vec('u8', Array.from(fromHEX(this.getAuthKey()))),
        Args.vec('address', addrs),
        Args.vec('string', mods),
        Args.vec('string', fns),
        Args.u64(BigInt(this.maxInactiveInterval)),
      ],
      info: `Welcome to ${this.appName}\nYou will authorize session:\n${
        'Scope:\n' + this.scopes.join('\n') + '\nTimeOut:' + this.maxInactiveInterval.toString()
      }`,
    })

    const result = await client.signAndExecuteTransaction({
      transaction: tx,
      signer: signer,
    })

    if (result.execution_info.status.type === 'executed') {
      return this
    } else {
      throw Error(`create session failed ${result.execution_info.status}`)
    }
  }

  toJSON(): any {
    return {
      appName: this.appName,
      appUrl: this.appUrl,
      scopes: this.scopes,
      secretKey: this.keypair.getSecretKey(),
      maxInactiveInterval: this.maxInactiveInterval,
      bitcoinAddress: this.bitcoinAddress.toStr(),
      roochAddress: this.roochAddress.toStr(),
      localCreateSessionTime: this.localCreateSessionTime,
      lastActiveTime: this.lastActiveTime,
    }
  }
}
