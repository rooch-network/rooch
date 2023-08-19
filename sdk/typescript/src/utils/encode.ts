import { ROOCH_ADDRESS_LENGTH } from '../constants'
import { FunctionId, AccountAddress, Identifier, TypeTag } from '../types'

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

export function typeTagToString(type_tag: TypeTag): string {
    if (typeof type_tag === 'string') {
        return type_tag
    } 
    
    if ('Vector' in type_tag) {
        return `Vector<${typeTagToString(type_tag.Vector)}>`
    } 
    
    if ('Struct' in type_tag) {
        const struct = type_tag.Struct
        let result = `${struct.address}::${struct.module}::${struct.name}`
        if (struct.type_params) {
            const params = struct.type_params.map(typeTagToString).join(', ')
            result += `<${params}>`
        }
        return result
    } 

    throw new Error(`Unknown type tag: ${JSON.stringify(type_tag)}`)
}