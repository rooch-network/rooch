import { Arg, TypeTag, FunctionId, FunctionReturnValue, Bytes } from '../types'

export interface IProvider {
  getRpcApiVersion(): Promise<string | undefined>

  executeViewFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
  ): Promise<FunctionReturnValue[]>

  sendRawTransaction(playload: Bytes): Promise<string>
}
