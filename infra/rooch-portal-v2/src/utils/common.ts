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
