import { FunctionId, TypeTag, functionIdToStirng, FunctionReturnValue } from '../types'

export interface IProvider {
    getRpcApiVersion(): Promise<string | undefined>;

    executeViewFunction(
        funcId: FunctionId,
        args?: Uint8Array[],
        tyArgs?: string[],
    ): Promise<FunctionReturnValue[]>
}