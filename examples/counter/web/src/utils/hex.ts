import { Buffer } from "buffer"

export function toHexString(byteArray: Iterable<number>): string {
    return '0x' + Buffer.from(new Uint8Array(byteArray)).toString('hex');
}

export function fromHexString(hex: string, padding?: number): Uint8Array {
    if (hex.startsWith('0x')) {
        hex = hex.substring(2);
    }
    if (padding) {
        if (hex.length < padding) {
            hex = padLeft(hex, padding);
        }
    } else {
        if (hex.length % 2 != 0) {
            hex = '0' + hex;
        }
    }
    const buf = Buffer.from(hex, 'hex');
    return new Uint8Array(buf);
}

/**
 * @public
 * Should be called to pad string to expected length
 */
export function padLeft(str: string, chars: number, sign?: string) {
    return new Array(chars - str.length + 1).join(sign ? sign : '0') + str;
}

/**
 * @public
 * Should be called to pad string to expected length
 */
export function padRight(str: string, chars: number, sign?: string) {
    return str + new Array(chars - str.length + 1).join(sign ? sign : '0');
}
