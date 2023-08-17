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

export const ROOCH_ADDRESS_LENGTH = 64
export type FunctionReturnValue = AnnotatedFunctionReturnValueView;