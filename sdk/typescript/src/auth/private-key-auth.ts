// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types'
import { Keypair } from '../utils/crypto'
import { IAuthorization, IAuthorizer } from './interface'

export const SCHEME_ED25519: number = 0

export class PrivateKeyAuth implements IAuthorizer {
  private pk: Keypair

  constructor(pk: Keypair) {
    this.pk = pk
  }

  async auth(data: Bytes): Promise<IAuthorization> {
    const sign = await this.pk.signMessage(data)

    return {
      scheme: SCHEME_ED25519,
      payload: sign.signature,
    }
  }
}
