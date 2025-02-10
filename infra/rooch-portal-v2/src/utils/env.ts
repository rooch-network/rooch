'use client';

const MAIN_NETWORK_DOMAINS = ['portal.rooch.network', 'main-portal.rooch.network'];

const isDevMode = process.env.NODE_ENV !== 'production';

export function isMainNetwork(): boolean {
  if (isDevMode) {
    // dev mode, can custom network,
    // true is mainnet
    // false is testnet
    return true;
  }
  return MAIN_NETWORK_DOMAINS.includes(window?.location?.hostname ?? '');
}
