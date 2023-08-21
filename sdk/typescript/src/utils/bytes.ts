import { Seq } from '../generated/runtime/serde/mod'

export function uint8Array2SeqNumber(bytes: Uint8Array): Seq<number> {
    return Array.from(bytes, byte => byte);
}