// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Keypair } from '../utils/crypto'
import { IAuthorization, IAuthorizer } from './interface'

const SCHEME_ED25519: number = 0

export class PrivateKeyAuth implements IAuthorizer {
  private pk: Keypair

  constructor(pk: Keypair) {
    this.pk = pk
  }

  async auth(data: Uint8Array): Promise<IAuthorization> {
    const sign = await this.pk.signMessage(data)

    return {
      scheme: SCHEME_ED25519,
      payload: sign.signature,
    }
  }
}
