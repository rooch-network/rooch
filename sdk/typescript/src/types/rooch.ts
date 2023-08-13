import {
  // AnnotatedEventView,
  AnnotatedFunctionReturnValueView,
  // AnnotatedStateView,
  // EventFilterView,
  // FunctionCallView,
  // PageView_for_Nullable_AnnotatedEventView_and_uint64,
  // PageView_for_Nullable_AnnotatedStateView_and_alloc_vec_Vec_U8Array,
  // PageView_for_Nullable_StateView_and_alloc_vec_Vec_U8Array,
  // PageView_for_Nullable_TransactionExecutionInfoView_and_uint128,
  // StateView,
  // TransactionExecutionInfoView,
  // TransactionView,
} from '../generated/client/types'

export type Identifier = string
export type AccountAddress = string
export type HashValue = string
export type U8 = number
export type U16 = number
export type U64 = number
export type U128 = number
export type U256 = string
export type I64 = number
export type BlockNumber = number
export type AuthenticationKey = string
export type MultiEd25519PublicKey = string
export type MultiEd25519Signature = string
export type EventKey = string

export type ModuleId = string | { address: AccountAddress; name: Identifier }
export type FunctionId =
  | string
  | { address: AccountAddress; module: Identifier; functionName: Identifier }

export interface StructTag {
  address: string
  module: string
  name: string
  // eslint-disable-next-line no-use-before-define
  type_params?: TypeTag[]
}

export type TypeTag =
  | 'Bool'
  | 'U8'
  | 'U64'
  | 'U128'
  | 'Address'
  | 'Signer'
  | { Vector: TypeTag }
  | { Struct: StructTag }

export function functionIdToStirng(functionId: FunctionId): string {
  if (typeof functionId !== 'string') {
    if (functionId instanceof Object) {
      return `${functionId.address}::${functionId.module}::${functionId.functionName}`
    }
  }
  return functionId
}

export function parseFunctionId(functionId: FunctionId): {
  address: AccountAddress
  module: Identifier
  functionName: Identifier
} {
  if (typeof functionId !== 'string') {
    return functionId
  }
  const parts = functionId.split('::', 3)

  if (parts.length !== 3) {
    throw new Error(`cannot parse ${functionId} into FunctionId`)
  }

  return {
    address: parts[0],
    module: parts[1],
    functionName: parts[2],
  }
}

export const ROOCH_ADDRESS_LENGTH = 64

/**
 * Perform the following operations:
 * 1. Make the address lower case
 * 2. Prepend `0x` if the string does not start with `0x`.
 * 3. Add more zeros if the length of the address(excluding `0x`) is less than `Rooch_ADDRESS_LENGTH`
 *
 * WARNING: if the address value itself starts with `0x`, e.g., `0x0x`, the default behavior
 * is to treat the first `0x` not as part of the address. The default behavior can be overridden by
 * setting `forceAdd0x` to true
 *
 */
export function normalizeRoochAddress(
  value: string,
  forceAdd0x: boolean = false,
): string {
  let address = value.toLowerCase()
  if (!forceAdd0x && address.startsWith('0x')) {
    address = address.slice(2)
  }
  return `0x${address.padStart(ROOCH_ADDRESS_LENGTH, '0')}`
}

export type FunctionReturnValue = AnnotatedFunctionReturnValueView;