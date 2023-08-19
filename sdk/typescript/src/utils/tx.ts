import { fromHexString } from './hex'
import { ROOCH_ADDRESS_LENGTH } from '../constants'
import * as rooch_types from '../generated/runtime/rooch_types/mod'
import { bytes as Bytes, Seq, Tuple, ListTuple, uint8} from '../generated/runtime/serde/mod'
import { BcsSerializer } from '../generated/runtime/bcs/mod'
import {
  AccountAddress,
  FunctionId,
  TypeTag,
  StructTag,
  Arg,
} from '../types'
import {parseFunctionId} from './encode'

export function encodeFunctionCall(
  functionId: FunctionId,
  tyArgs: TypeTag[],
  args: Bytes[],
): rooch_types.MoveActionVariantFunction {
  const funcId = parseFunctionId(functionId)

  const functionCall = new rooch_types.FunctionCall(
    new rooch_types.FunctionId(
      new rooch_types.ModuleId(
        addressToSCS(funcId.address),
        new rooch_types.Identifier(funcId.module),
      ),
      new rooch_types.Identifier(funcId.functionName),
    ),
    tyArgs.map((t) => typeTagToSCS(t)),
    bytesArrayToSeqSeq(args),
  )

  return new rooch_types.MoveActionVariantFunction(functionCall)
}

export function typeTagToSCS(ty: TypeTag): rooch_types.TypeTag {
  if (ty === 'Bool') {
    return new rooch_types.TypeTagVariantbool()
  }
  if (ty === 'U8') {
    return new rooch_types.TypeTagVariantu8()
  }
  if (ty === 'U128') {
    return new rooch_types.TypeTagVariantu128()
  }
  if (ty === 'U64') {
    return new rooch_types.TypeTagVariantu64()
  }
  if (ty === 'Address') {
    return new rooch_types.TypeTagVariantaddress()
  }
  if (ty === 'Signer') {
    return new rooch_types.TypeTagVariantsigner()
  }
  if ('Vector' in ty) {
    return new rooch_types.TypeTagVariantvector(typeTagToSCS(ty.Vector))
  }
  if ('Struct' in ty) {
    return new rooch_types.TypeTagVariantstruct(structTagToSCS(ty.Struct))
  }
  throw new Error(`invalid type tag: ${ty}`)
}

export function structTagToSCS(data: StructTag): rooch_types.StructTag {
  return new rooch_types.StructTag(
    addressToSCS(data.address),
    new rooch_types.Identifier(data.module),
    new rooch_types.Identifier(data.name),
    data.type_params ? data.type_params.map((t) => typeTagToSCS(t)) : [],
  )
}

export function addressToSCS(addr: AccountAddress): rooch_types.AccountAddress {
  // AccountAddress should be 16 bytes, in hex, it's 16 * 2.
  const bytes = fromHexString(addr, 16 * 2)
  const data: [number][] = []
  for (let i = 0; i < bytes.length; i++) {
    data.push([bytes[i]])
  }
  return new rooch_types.AccountAddress(data)
}

export function encodeStructTypeTags(typeArgsString: string[]): TypeTag[] {
  return typeArgsString.map((str) => encodeStructTypeTag(str))
}

function encodeStructTypeTag(str: string): TypeTag {
  const arr = str.split('<')
  const arr1 = arr[0].split('::')
  const address = arr1[0]
  const module = arr1[1]
  const name = arr1[2]

  const params = arr[1] ? arr[1].replace('>', '').split(',') : []
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const type_params: TypeTag[] = []
  if (params.length > 0) {
    params.forEach((param: string) => {
      type_params.push(encodeStructTypeTag(param.trim()))
    })
  }

  const result: TypeTag = {
    Struct: {
      address,
      module,
      name,
      type_params,
    },
  }
  return result
}

function bytesArrayToSeqSeq(input: Bytes[]): Seq<Seq<number>> {
  return input.map((byteArray) => Array.from(byteArray))
}

export function addressToListTuple(ethAddress: string): ListTuple<[uint8]> {
  // Remove '0x' prefix
  const cleanedEthAddress = ethAddress.startsWith('0x') ? ethAddress.slice(2) : ethAddress

  // Check if the address is valid
  if (cleanedEthAddress.length !== ROOCH_ADDRESS_LENGTH) {
      throw new Error('Invalid Ethereum address')
  }

  // Convert to list of tuples
  const listTuple: ListTuple<[uint8]> = []
  for (let i = 0; i < cleanedEthAddress.length; i += 2) {
      const byte = parseInt(ethAddress.slice(i, i + 2), 16)
      listTuple.push([byte] as Tuple<[uint8]>)
  }

  return listTuple
}


export function encodeArgs(arg: Arg): Bytes {
  const se = new BcsSerializer()

  if (arg.type === 'Bool') {
    se.serializeBool(arg.value as boolean)
  } else if (arg.type === 'U8') {
    se.serializeU8(arg.value as number)
  } else if (arg.type === 'U64') {
    se.serializeU64(arg.value as bigint | number)
  } else if (arg.type === 'U128') {
    se.serializeU128(arg.value as bigint | number)
  } else if (arg.type === 'Address') {
    const list = addressToListTuple(arg.value as string)
    const accountAddress = new rooch_types.AccountAddress(list)
    accountAddress.serialize(se)
  }

  return se.getBytes()
}

export const encodeMoveCallData = (
  funcId: FunctionId,
  tyArgs: TypeTag[],
  args: Arg[])=>{
  const bcsArgs = args?.map(arg=>encodeArgs(arg))
  const scriptFunction = encodeFunctionCall(funcId, tyArgs, bcsArgs)

  const payloadInHex = (() => {
    const se = new BcsSerializer()
    scriptFunction.serialize(se)
    return se.getBytes()
  })()

  return payloadInHex
}