export type Identifier = string;
export type AccountAddress = string;
export type HashValue = string;
export type U8 = number;
export type U16 = number;
export type U64 = number;
export type U128 = number;
export type U256 = string;
export type I64 = number;
export type BlockNumber = number;
export type AuthenticationKey = string;
export type MultiEd25519PublicKey = string;
export type MultiEd25519Signature = string;
export type EventKey = string;

export type ModuleId = string | { address: AccountAddress; name: Identifier };
export type FunctionId =
    | string
    | { address: AccountAddress; module: Identifier; functionName: Identifier };

export interface StructTag {
    address: string;
    module: string;
    name: string;
    // eslint-disable-next-line no-use-before-define
    type_params?: TypeTag[];
}

export type TypeTag =
    | 'Bool'
    | 'U8'
    | 'U64'
    | 'U128'
    | 'Address'
    | 'Signer'
    | { Vector: TypeTag }
    | { Struct: StructTag };

export function parseFunctionId(
    functionId: FunctionId
): { address: AccountAddress; module: Identifier; functionName: Identifier } {
    if (typeof functionId !== 'string') {
        return functionId;
    } else {
        const parts = functionId.split('::', 3);
        if (parts.length !== 3) {
            throw new Error(`cannot parse ${functionId} into FunctionId`);
        }
        return {
            address: parts[0],
            module: parts[1],
            functionName: parts[2],
        };
    }
}
