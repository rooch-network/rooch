import { shortAddress } from './address';

export const isSessionExpired = (lastActiveTime: number, maxInactiveInterval: number) => {
  const expirationTime = (lastActiveTime + maxInactiveInterval) * 1000;
  return Date.now() > expirationTime;
};

export function sleep(time: number) {
  return new Promise((resolve) => setTimeout(resolve, time));
}

export const hexToString = (hex: string): string => {
  if (hex.startsWith('0x')) {
    hex = hex.substring(2);
  }

  let result = '';
  for (let i = 0; i < hex.length; i += 2) {
    const byte = parseInt(hex.substring(i, i + 2), 16);
    result += String.fromCharCode(byte);
  }

  return result;
};

export function shortCoinType(
  coinType: string | `${string}::${string}::${string}`,
  start = 6,
  end = 4
): string {
  try {
    if (!coinType) {
      return '';
    }
    const temp = coinType.split('::');
    return `${shortAddress(temp[0], 5, 0)}...::${temp[2]}`;
  } catch (error) {
    return '';
  }
}
