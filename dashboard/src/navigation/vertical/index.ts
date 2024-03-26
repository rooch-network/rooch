// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { VerticalNavItemsType } from 'src/@core/layouts/types'

const navigation = (): VerticalNavItemsType => {
  return [
    {
      sectionTitle: 'Scan',
    },
    {
      title: 'State',
      icon: 'bxs-tree',
      children: [
        {
          title: 'Get',
          path: '/scan/state/get',
        },
        {
          title: 'List',
          path: '/scan/state/list',
        },
      ],
    },
    {
      title: 'Transaction',
      icon: 'bx-collection',
      path: '/scan/transaction/list',
    },
    {
      sectionTitle: 'Assets',
    },

    // {
    //   title: 'Wallet',
    //   icon: 'bx-wallet',
    //   path: '/wallet',
    // },
    {
      title: 'My Assets',
      icon: 'bxs-badge-dollar',
      children: [
        {
          title: 'Inscription',
          path: '/assets/inscription/list',
        },
        {
          title: 'UTXO',
          path: '/assets/utxo/list',
        },
        {
          title: 'Mint',
          externalLink: true,
          openInNewTab: true,
          path: 'https://inscribetheplanet.com/testnet',
        },
      ],
    },
    {
      sectionTitle: 'Tutorial',
      domain: 'https://dev-dashboard.rooch.network',
    },
    {
      title: 'Counter Example',
      icon: 'bxs-package',
      path: '/tutorial/counter',
      domain: 'https://dev-dashboard.rooch.network',
    },

    // {
    //   sectionTitle: 'Authentication',
    // },
    // {
    //   title: 'Session',
    //   icon: 'bx:food-menu',
    //   path: '/session',
    // },
  ]
}

export default navigation
