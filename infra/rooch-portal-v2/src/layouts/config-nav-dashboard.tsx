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
        icon: <Iconify icon="solar:user-id-bold-duotone" />,
      },
      {
        title: 'Assets',
        path: paths.dashboard.assets,
        icon: <Iconify icon="solar:wallet-money-bold-duotone" />,
      },
      {
        title: 'Transactions',
        path: paths.dashboard.transactions,
        icon: <Iconify icon="solar:clipboard-list-bold-duotone" />,
      },
      {
        title: 'Purchase Gas',
        path: paths.dashboard['gas-swap'],
        icon: <Iconify icon="solar:gas-station-bold-duotone" />,
        noAddressRequired: true,
      },
      {
        title: 'Faucet',
        path: paths.dashboard.faucet,
        icon: <Iconify icon="solar:gift-bold-duotone" />,
        // noAddressRequired: true,
      },
      {
        title: 'Settings',
        path: paths.dashboard.settings,
        icon: <Iconify icon="solar:settings-bold-duotone" />,
        noAddressRequired: true,
        connectWalletRequired: false,
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
        icon: <Iconify icon="solar:card-search-bold-duotone" />,
        noAddressRequired: true,
      },
      {
        title: 'Apps',
        path: paths.dashboard.apps,
        icon: <Iconify icon="solar:widget-5-bold-duotone" />,
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
    ].filter((item) => !(isMainNetwork() && item.title === 'Apps')),
  },
].filter((item) => !(isMainNetwork() && item.subheader === 'Tokens'));
