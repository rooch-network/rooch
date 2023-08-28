import {
  Arg,
  TypeTag,
  FunctionId,
  AnnotatedFunctionResultView,
  Bytes,
} from '../types'

export interface IProvider {
  getRpcApiVersion(): Promise<string | undefined>

  getChainId(): number

  executeViewFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
  ): Promise<AnnotatedFunctionResultView>

  sendRawTransaction(playload: Bytes): Promise<string>
}
