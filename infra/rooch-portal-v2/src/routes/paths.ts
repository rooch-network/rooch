const ROOTS = {
  DASHBOARD: '',
};

export const paths = {
  dashboard: {
    root: ROOTS.DASHBOARD,
    account: `${ROOTS.DASHBOARD}/account`,
    assets: `${ROOTS.DASHBOARD}/assets`,
    mint: `${ROOTS.DASHBOARD}/mint`,
    transactions: `${ROOTS.DASHBOARD}/transactions`,
    market: `${ROOTS.DASHBOARD}/trade/market`,
    liquidity: `${ROOTS.DASHBOARD}/trade/liquidity`,
    swap: `${ROOTS.DASHBOARD}/trade/swap`,
    'swap-v2': `${ROOTS.DASHBOARD}/trade/swap-v2`,
    history: `${ROOTS.DASHBOARD}/history`,
    apps: `${ROOTS.DASHBOARD}/apps`,
    settings: `${ROOTS.DASHBOARD}/settings`,
    search: `${ROOTS.DASHBOARD}/search`,
    faucet: `${ROOTS.DASHBOARD}/faucet`,
    invitation: `${ROOTS.DASHBOARD}/invitation`,
    'gas-swap': `${ROOTS.DASHBOARD}/gas-swap`,
  },
};
