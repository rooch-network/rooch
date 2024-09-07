export function extractCoinType(coinTypeWithWrap?: string) {
  if (!coinTypeWithWrap) {
    return undefined;
  }
  const startIndex = coinTypeWithWrap.indexOf('0x3::coin_store::CoinStore<');
  const endIndex = coinTypeWithWrap.lastIndexOf('>');
  return coinTypeWithWrap.substring(startIndex, endIndex);
}
