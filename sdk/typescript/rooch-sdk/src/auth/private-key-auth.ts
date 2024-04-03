// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Keypair } from '../utils/crypto'
import { IAuthorizer } from './interface'
import { Authenticator } from '../generated/runtime/rooch_types/mod'
import { uint8Array2SeqNumber } from '../utils'
import { runtime } from '../index'

const SCHEME_ED25519: number = 0

export class PrivateKeyAuth implements IAuthorizer {
  private pk: Keypair

  constructor(pk: Keypair) {
    this.pk = pk
  }

  async auth(data: Uint8Array, _?: string): Promise<runtime.Authenticator> {
    const sign = await this.pk.signMessage(data)

    return new Authenticator(BigInt(SCHEME_ED25519), uint8Array2SeqNumber(sign.signature))
  }
}
