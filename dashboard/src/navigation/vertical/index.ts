// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { VerticalNavItemsType } from 'src/@core/layouts/types'

const navigation = (): VerticalNavItemsType => {
  return [
    {
      title: 'Dashboards',
      icon: 'bx:home-circle',
      children: [
        {
          title: 'Analytics',
          path: '/dashboards/analytics',
        },
      ],
    },

    // {
    //   sectionTitle: 'Tutorial',
    // },
    // {
    //   title: 'Publish Package',
    //   icon: 'bxs-package',
    //   path: '/tutorial/publish/package',
    // },
    {
      sectionTitle: 'Scan',
    },
    {
      title: 'State',
      icon: 'bxs-tree',
      path: '/scan/state/list',
    },
    {
      title: 'Transaction',
      icon: 'bx-collection',
      path: '/scan/transaction/list',
    },

    // {
    //   sectionTitle: 'Wallet & Assets',
    // },
    // {
    //   title: 'Wallet',
    //   icon: 'bx-wallet',
    //   path: '/wallet',
    // },
    // {
    //   title: 'Assets',
    //   icon: 'bxs-badge-dollar',
    //   children: [
    //     {
    //       title: 'Overvier',
    //       path: '/assets/overview',
    //     },
    //     {
    //       title: 'Deposit',
    //       path: '/assets/deposit',
    //     },
    //     {
    //       title: 'Withdraw',
    //       path: '/assets/withdraw',
    //     },
    //     {
    //       title: 'Transfer',
    //       path: '/assets/transfer',
    //     },
    //   ],
    // },
    {
      sectionTitle: 'Authentication',
    },
    {
      title: 'Session',
      icon: 'bx:food-menu',
      path: '/session',
    },

    // {
    //   title: 'OAuth',
    //   icon: 'bx:lock-open-alt',
    //   path: '/oauth',
    // },
    {
      sectionTitle: 'Other',
    },
    {
      title: 'Setting',
      icon: 'bx:dots-horizontal-rounded',
      path: '/setting',
    },
  ]
}

export default navigation
