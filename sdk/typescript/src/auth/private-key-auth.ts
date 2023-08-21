import { Bytes } from '../types'
import { Keypair } from '../utils/crypto'
import { IAuthorization, IAuthorizer } from './interface'

export const SCHEME_ED25519: number = 0

export class PrivateKeyAuth implements IAuthorizer {
    private pk: Keypair

    constructor(pk: Keypair) {
        this.pk = pk
    }

    auth(data: Bytes): IAuthorization {
        const signData = this.pk.signData(data)

        return {
            scheme: SCHEME_ED25519,
            payload: signData
        }
    }
}