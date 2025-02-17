import { paths } from 'src/routes/paths';

import { Iconify } from 'src/components/iconify';

import { isMainNetwork } from '../utils/env';

export const navData = [
  /**
   * My Account
   */
  {
    subheader: 'My Account',
    items: [
      {
        title: 'Account',
        path: paths.dashboard.account,
        icon: <Iconify icon="solar:user-id-broken" />,
      },
      {
        title: 'Assets',
        path: paths.dashboard.assets,
        icon: <Iconify icon="solar:wallet-2-broken" />,
      },
      {
        title: 'Transactions',
        path: paths.dashboard.transactions,
        icon: <Iconify icon="solar:list-up-broken" />,
      },
      {
        title: 'Purchase Gas',
        path: paths.dashboard['gas-swap'],
        icon: <Iconify icon="solar:gas-station-broken" />,
        noAddressRequired: true,
      },
      {
        title: 'Faucet',
        path: paths.dashboard.faucet,
        icon: <Iconify icon="solar:gift-broken" />,
      },
      {
        title: 'Invitation',
        path: paths.dashboard.invitation,
        icon: <Iconify icon="solar:letter-broken" />,
      },
      {
        title: 'Settings',
        path: paths.dashboard.settings,
        icon: <Iconify icon="solar:settings-broken" />,
        noAddressRequired: true,
        connectWalletRequired: false,
      },
    ],
  },
  /**
   * Tokens
   */
  {
    subheader: 'Trade',
    items: [
      {
        title: 'Marketplace',
        path: paths.dashboard.market,
        icon: <Iconify icon="solar:cart-large-2-broken" />,
        noAddressRequired: true,
      },
      {
        title: 'Liquidity',
        path: paths.dashboard.liquidity,
        icon: <Iconify icon="solar:hand-money-broken" />,
        noAddressRequired: true,
      },
      // temporary disable swap, when the swap v2 is all good, will remove this path
      {
        title: 'Swap',
        path: paths.dashboard['swap-v2'],
        icon: <Iconify icon="solar:money-bag-broken" />,
        noAddressRequired: true,
      },
    ],
  },
  /**
   * Tokens
   */
  {
    subheader: 'Tokens',
    items: [
      {
        title: 'Mint',
        path: paths.dashboard.mint,
        icon: <Iconify icon="solar:star-fall-bold-duotone" />,
        noAddressRequired: true,
      },
    ],
  },
  /**
   * Tools
   */
  {
    subheader: 'Tools',
    items: [
      {
        title: 'Search',
        path: paths.dashboard.search,
        icon: <Iconify icon="solar:card-search-broken" />,
        noAddressRequired: true,
      },
      {
        title: 'Apps',
        path: paths.dashboard.apps,
        icon: <Iconify icon="solar:widget-5-broken" />,
        noAddressRequired: true,
      },
      {
        title: isMainNetwork() ? 'TestNet' : 'MainNet',
        path: isMainNetwork()
          ? 'https://test-portal.rooch.network'
          : 'https://portal.rooch.network',
        icon: <Iconify icon="solar:infinity-line-duotone" />,
        noAddressRequired: true,
      },
    ], // .filter((item) => !(isMainNetwork() && item.title === 'Apps')),
  },
].filter((item) => !(isMainNetwork() && item.subheader === 'Tokens'));
// .filter((item) => !(isMainNetwork() && (item.subheader === 'Tokens' || item.subheader === 'Trade')));
