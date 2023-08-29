import { Bytes } from '../types'
import { Keypair } from '../utils/crypto'
import { IAuthorization, IAuthorizer } from './interface'

const SCHEME_ED25519: number = 3

export class SessionKeyAuth implements IAuthorizer {
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
