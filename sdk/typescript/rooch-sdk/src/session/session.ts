// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Args } from '@/bcs'
import { RoochClient } from '@/client'
import { Ed25519Keypair } from '@/keypairs'
import { Transaction } from '@/transactions'
import { BitcoinAddress, normalizeRoochAddress, RoochAddress } from '@/address'
import { Authenticator, PublicKey, SignatureScheme, Signer } from '@/crypto'

import { CreateSessionArgs, Scope } from './types'

const DEFAULT_MAX_INACTIVE_INTERVAL = 1200 // second
const REQUIRED_SCOPE = `${normalizeRoochAddress('0x3')}::session_key::remove_session_key_entry`

type InnerCreateSessionArgs = {
  client: RoochClient
  signer: Signer
} & CreateSessionArgs

export class Session extends Signer {
  protected readonly appName: string
  protected readonly appUrl: string
  protected readonly scopes: Scope[]
  protected readonly keypair: Ed25519Keypair
  protected readonly maxInactiveInterval: number
  protected readonly bitcoinAddress: BitcoinAddress
  protected readonly roochAddress: RoochAddress

  protected constructor(
    appName: string,
    appUrl: string,
    scopes: Scope[],
    roochAddress: RoochAddress,
    bitcoinAddress: BitcoinAddress,
    keypair?: Ed25519Keypair,
    maxInactiveInterval?: number,
    localCreateSessionTime?: number,
  ) {
    super()
    this.appName = appName
    this.appUrl = appUrl
    this.scopes = scopes
    this.roochAddress = roochAddress
    this.bitcoinAddress = bitcoinAddress
    this.keypair = keypair ?? Ed25519Keypair.generate()
    this.maxInactiveInterval = maxInactiveInterval ?? DEFAULT_MAX_INACTIVE_INTERVAL
    this.localCreateSessionTime = localCreateSessionTime ?? Date.now() / 1000
  }

  protected readonly localCreateSessionTime: number

  public static async CREATE(input: InnerCreateSessionArgs): Promise<Session> {
    const parsedScopes = input.scopes.map((scope) => {
      if (typeof scope === 'string') {
        const [pkg, mod, fn] = scope.split('::')
        return {
          address: pkg,
          module: mod,
          function: fn,
        }
      }
      return scope
    })

    const allOx3 = `${normalizeRoochAddress('0x3')}::*::*`

    if (
      !parsedScopes
        .map((item) => `${normalizeRoochAddress(item.address)}::${item.module}::${item.function}`)
        .find((item) => item === allOx3 || REQUIRED_SCOPE)
    ) {
      const [pkg, mod, fn] = REQUIRED_SCOPE.split('::')
      parsedScopes.push({
        address: pkg,
        module: mod,
        function: fn,
      })
    }

    return new Session(
      input.appName,
      input.appUrl,
      parsedScopes,
      input.signer.getRoochAddress(),
      input.signer.getBitcoinAddress(),
      input.keypair,
      input.maxInactiveInterval,
    ).build(input.client, input.signer)
  }

  sign(input: Uint8Array): Promise<Uint8Array> {
    return this.keypair.sign(input)
  }

  protected signTransactionImp(input: Uint8Array): Promise<Authenticator> {
    return Authenticator.rooch(input, this)
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

  getAuthKey(): RoochAddress {
    return this.keypair.getRoochAddress()
  }

  protected async build(client: RoochClient, signer: Signer) {
    const [addrs, mods, fns] = this.scopes
      .map((scope) => {
        return [scope.address, scope.module, scope.function]
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
      arguments: [
        Args.string(this.appName),
        Args.string(this.appUrl),
        Args.vec('u8', Array.from(this.getAuthKey().toBytes())),
        Args.vec('address', addrs),
        Args.vec('string', mods),
        Args.vec('string', fns),
        Args.u64(BigInt(this.maxInactiveInterval)),
      ],
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
}
