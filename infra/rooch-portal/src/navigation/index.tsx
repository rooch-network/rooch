// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  Bitcoin,
  LucideIcon,
  Compass,
  LayoutGrid,
  Scroll,
  UserCog,
  CandlestickChart,
  ArrowLeftRight,
} from 'lucide-react'
import * as React from 'react'
import { Routes, Route } from 'react-router-dom'
import { AssetsLayout } from '@/pages/assets/assets-layout.tsx'
import { MintLayout } from '@/pages/mint/layout.tsx'
import { TradeLayout } from '@/pages/trade/trade-layout.tsx'
import { LeapLayout } from '@/pages/leap/leap-layout.tsx'
import { MintDetailLayout } from '@/pages/mint/detail/layout.tsx'
import { TransactionsLayout } from '@/pages/transactions/layout.tsx'
import { TransactionDetailLayout } from '@/pages/transactions/detail/layout'
import { AppsLayout } from '@/pages/apps/apps-layout.tsx'
import { SettingsLayout } from '@/pages/settings/settings-layout.tsx'

export type NavLink = {
  icon: LucideIcon
  label: string
  path: string
  auth?: boolean
  disabled?: boolean
  element?: React.ReactElement
}

export type NavItemsType = NavLink[]

export const allRouter = [
  { path: '/', element: <AssetsLayout /> },
  { path: '/mint', element: <MintLayout /> },
  { path: '/mint/detail/:address', element: <MintDetailLayout /> },
  { path: '/trade', element: <TradeLayout /> },
  { path: '/leap', element: <LeapLayout /> },
  { path: '/apps', element: <AppsLayout /> },
  { path: '/transactions', element: <TransactionsLayout /> },
  { path: '/transactions/detail/:hash', element: <TransactionDetailLayout /> },
  { path: '/settings', element: <SettingsLayout /> },
]

export const navItems: NavItemsType = [
  { icon: Bitcoin, label: 'Sidebar.assets', path: '/', auth: true, element: <AssetsLayout /> },
  { icon: Scroll, label: 'Sidebar.mint', path: '/mint', auth: true, element: <MintLayout /> },
  {
    icon: CandlestickChart,
    label: 'Sidebar.trade',
    path: '/trade',
    element: <TradeLayout />,
  },
  {
    icon: ArrowLeftRight,
    label: 'Sidebar.leap',
    path: '/leap',
    element: <LeapLayout />,
  },
  {
    icon: Compass,
    label: 'Sidebar.transactions',
    path: '/transactions',
    element: <TransactionsLayout />,
  },
  { icon: LayoutGrid, label: 'Sidebar.apps', path: '/apps', element: <AppsLayout /> },
  {
    icon: UserCog,
    label: 'Sidebar.settings',
    path: '/settings',
    auth: true,
    element: <SettingsLayout />,
  },
]

export const routers = (): React.ReactElement => {
  return (
    <Routes>
      {allRouter.map((item) => (
        <Route key={item.path} path={item.path} element={item.element} />
      ))}
    </Routes>
  )
}
