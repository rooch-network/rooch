import { AccountAddress, FunctionId, TypeTag, StructTag, parseFunctionId } from '../types';

import * as rooch_types from "../lib/runtime/rooch_types/mod"
import { bytes } from '../lib/runtime/serde/mod';
import { BcsDeserializer } from "../lib/runtime/bcs/mod";
import { fromHexString } from './hex';

export function encodeFunctionCall(functionId: FunctionId, tyArgs: TypeTag[], args: bytes[]): rooch_types.MoveActionVariantFunction {
    const funcId = parseFunctionId(functionId)

    const functionCall = new rooch_types.FunctionCall(
        new rooch_types.FunctionId(
            new rooch_types.ModuleId(
                addressToSCS(funcId.address),
                new rooch_types.Identifier(funcId.module)
            ),
            new rooch_types.Identifier(funcId.functionName)
        ),
        tyArgs.map((t) => typeTagToSCS(t)),
        args
    )

    return new rooch_types.MoveActionVariantFunction(functionCall)
}

export function typeTagToSCS(ty: TypeTag): rooch_types.TypeTag {
    if (ty === 'Bool') {
        return new rooch_types.TypeTagVariantbool();
    }
    if (ty === 'U8') {
        return new rooch_types.TypeTagVariantu8();
    }
    if (ty === 'U128') {
        return new rooch_types.TypeTagVariantu128();
    }
    if (ty === 'U64') {
        return new rooch_types.TypeTagVariantu64();
    }
    if (ty === 'Address') {
        return new rooch_types.TypeTagVariantaddress();
    }
    if (ty === 'Signer') {
        return new rooch_types.TypeTagVariantsigner();
    }
    if ('Vector' in ty) {
        return new rooch_types.TypeTagVariantvector(typeTagToSCS(ty.Vector));
    }
    if ('Struct' in ty) {
        return new rooch_types.TypeTagVariantstruct(structTagToSCS(ty.Struct));
    }
    throw new Error(`invalid type tag: ${ty}`);
}

export function structTagToSCS(data: StructTag): rooch_types.StructTag {
    return new rooch_types.StructTag(
        addressToSCS(data.address),
        new rooch_types.Identifier(data.module),
        new rooch_types.Identifier(data.name),
        data.type_params ? data.type_params.map((t) => typeTagToSCS(t)) : []
    );
}

export function addressToSCS(
    addr: AccountAddress
): rooch_types.AccountAddress {
    // AccountAddress should be 16 bytes, in hex, it's 16 * 2.
    const bytes = fromHexString(addr, 16 * 2);
    return rooch_types.AccountAddress.deserialize(new BcsDeserializer(bytes));
}

export function encodeStructTypeTags(
    typeArgsString: string[]
): TypeTag[] {
    return typeArgsString.map((str) => encodeStructTypeTag(str))
}

function encodeStructTypeTag(
    str: string
): TypeTag {
    const arr = str.split('<');
    const arr1 = arr[0].split('::');
    const address = arr1[0];
    const module = arr1[1];
    const name = arr1[2];

    const params = arr[1] ? arr[1].replace('>', '').split(',') : [];
    // eslint-disable-next-line @typescript-eslint/naming-convention
    const type_params: TypeTag[] = [];
    if (params.length > 0) {
        params.forEach((param: string) => {
            type_params.push(encodeStructTypeTag(param.trim()));
        });
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