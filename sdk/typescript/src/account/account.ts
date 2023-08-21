import { IAccount } from './interface'
import { IProvider } from '../provider'
import { IAuthorizer } from '../auth'
import { AccountAddress, FunctionId, TypeTag, Arg } from '../types'
import { BcsSerializer } from '../generated/runtime/bcs/mod'
import {
    RoochTransaction,
    RoochTransactionData,
    AccountAddress as BCSAccountAddress,
    Authenticator
} from '../generated/runtime/rooch_types/mod'
import {
    encodeArgs,
    encodeFunctionCall,
    addressToListTuple,
    uint8Array2SeqNumber
} from '../utils'

export class Account implements IAccount {
    private provider: IProvider

    private address: AccountAddress

    private authorizer: IAuthorizer

    private sequenceNumber: bigint

    public constructor(provider: IProvider, address: AccountAddress, authorizer: IAuthorizer) {
        this.provider = provider
        this.address = address
        this.authorizer = authorizer
        this.sequenceNumber = BigInt('0')
    }

    public callFunction(
        funcId: FunctionId,
        tyArgs: TypeTag[],
        args: Arg[]
    ): Promise<string> {
        const bcsArgs = args.map(arg => encodeArgs(arg))
        const scriptFunction = encodeFunctionCall(funcId, tyArgs, bcsArgs)
        const data = new RoochTransactionData(
            new BCSAccountAddress(addressToListTuple(this.address)),
            this.sequenceNumber,
            scriptFunction)

        const authPayload = this.makeAuth(data)
        const auth = new Authenticator(BigInt(authPayload.scheme), uint8Array2SeqNumber(authPayload.payload))
        const ts = new RoochTransaction(data, auth)

        const payload = (() => {
            const se = new BcsSerializer()
            ts.serialize(se)
            return se.getBytes()
        })()

        return this.provider.sendRawTransaction(payload)
    }

    private makeAuth(tsData: RoochTransactionData) {
        const payload = (() => {
            const se = new BcsSerializer()
            tsData.serialize(se)
            return se.getBytes()
        })()

        return this.authorizer.auth(payload)
    }
}