import BigNumber from 'bignumber.js';

export function formatUnitPrice(value: string, toCoinDecimals: number): string {
  return new BigNumber(value)
    .times(new BigNumber(10).pow(toCoinDecimals))
    .div(new BigNumber(10).pow(5))
    .toFixed();
}
