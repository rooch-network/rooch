import type { UserCoin } from 'src/components/swap/types';

import numeral from 'numeral';
import { BigNumber } from 'bignumber.js';

export function decimalsMultiplier(decimals?: BigNumber.Value) {
  return toBigNumber(10).pow(toBigNumber(decimals).abs());
}

export function formatCoinAmount(amount: number, decimals: number): string {
  return toBigNumber(amount).div(decimalsMultiplier(decimals)).toFixed(decimals).toString();
}

export const splitValue = (value: string) => {
  const spiltValue_ = String(value).split('.');
  const decimalLength = spiltValue_[1] && spiltValue_[1].length;
  return {
    spiltValue_,
    lastIndex: Number(decimalLength) - 1,
  };
};

export function formatCurrency(amount: BigNumber.Value, decimals: number, suffix?: string) {
  let value = +(amount || 0) / 10 ** decimals;
  let fixed = decimals;
  let prefix = '';

  if (value > 10) {
    fixed = 2;
  } else if (value >= 1) {
    fixed = 4;
  } else if (value >= 0.1) {
    fixed = 6;
  }

  if (value === 0) {
    fixed = 0;
  } else if (value < 1 / 10 ** decimals) {
    fixed = decimals;
    value = +Number(1 / 10 ** decimals);
    prefix = '~';
  }

  fixed = fixed > decimals ? decimals : fixed;

  const formatter = Intl.NumberFormat('en', {
    notation: 'standard',
    minimumFractionDigits: fixed,
    maximumFractionDigits: fixed,
  });

  return prefix + cutTrailingZerosFromString(formatter.format(value));
}

export function cutTrailingZerosFromString(numberAsString: string) {
  if (numberAsString.length === 1) return numberAsString;
  const arr = numberAsString.match(/^-?(((\d{1,3}),?)+\.*?)*?\d*?(?=\.?0*$)/) || [''];
  return arr[0];
}

BigNumber.config({
  EXPONENTIAL_AT: 1e9,
});

export function toDust(val: BigNumber | number | string, decimal: number | bigint): bigint {
  return bigNumberToBigInt(toDustBigNumber(val, decimal));
}

export function fromDust(
  val: BigNumber | bigint | number | string,
  decimal: number | bigint
): BigNumber {
  return toBigNumber(val).div(new BigNumber(10).pow(decimal.toString()));
}

function toDustBigNumber(val: BigNumber | number | string, decimal: number | bigint): BigNumber {
  return toBigNumber(val).times(new BigNumber(10).pow(decimal.toString()));
}

export function bigNumberToBigInt(val: BigNumber | number | string): bigint {
  const str = toBigNumber(val).toString();
  return BigInt(str);
}

export function bigIntToBigNumber(val: bigint) {
  return BigNumber(val.toString());
}

export function fromDustToPrecision(
  val: BigNumber | bigint | number | string,
  decimal: number | bigint
): string {
  BigNumber.config({ EXPONENTIAL_AT: 1e9 });
  const real = toBigNumber(val).div(new BigNumber(10).pow(decimal.toString()));
  // <1
  if (real.isLessThan(1)) {
    return real.toPrecision(3).replace(/\.?0+$/, '');
    // >=1 && <1,000,000
  }
  if (real.isGreaterThanOrEqualTo(1) && real.isLessThan(1000000)) {
    return real.toFixed(2);
    // >1,000,000
  }
  return real.toFixed(0);
}

export function formatCoin(coin: UserCoin, useBalance?: boolean) {
  return fromDustToPrecision(useBalance ? coin.balance : coin.amount, coin.decimals);
}

/**
 * Format value with given config
 *
 * ```ts
 * numberDecimalSeparator(500000)           // '500,000'
 * ```
 *
 * @param val format value.
 */
export function numberDecimalSeparator(
  val: BigNumber | bigint | number | string,
  groupSeparator?: string,
  decimalSeparator?: string
) {
  return BigNumber(val.toString())
    .toFormat({
      groupSize: 3,
      groupSeparator: groupSeparator ?? ',',
      decimalSeparator: decimalSeparator ?? '.',
    })
    .toString();
}

export function toBigNumber(
  val?: BigNumber | bigint | number | string | BigNumber.Instance
): BigNumber {
  if (val instanceof BigNumber || (BigNumber.isBigNumber(val) && typeof val === 'object')) {
    return val;
  }
  if (typeof val === 'number' || typeof val === 'string') {
    return BigNumber(val);
  }
  if (typeof val === 'bigint') {
    return bigIntToBigNumber(val);
  }
  if (typeof val === 'undefined') {
    return BigNumber(0);
  }
  return BigNumber(val);
}

export function currencyNumberFormatter(val: string | number | bigint) {
  const bn = new BigNumber(val.toString());
  const formatStringWithLessThanOne = '0.[000000000000000000]';
  const formatStringWithGreaterThanM = '0.00 a';
  const formatString = '0,0.00';
  if (bn.isLessThan(1)) {
    return numeral(bn.toNumber().toPrecision(3)).format(formatStringWithLessThanOne);
  }
  if (bn.isGreaterThanOrEqualTo(1e6)) {
    return numeral(bn.toNumber()).format(formatStringWithGreaterThanM);
  }
  return numeral(bn.toNumber()).format(formatString);
}
