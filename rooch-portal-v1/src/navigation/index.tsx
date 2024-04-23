import { Bitcoin, LucideIcon, Compass, LayoutGrid, Scroll, UserCog } from 'lucide-react'
import * as React from 'react'
import { Routes, Route } from 'react-router-dom'
import { AssetsLayout } from '@/pages/assets/assets-layout.tsx'
import { MintLayout } from '@/pages/mint/mint-layout.tsx'
import { AppsLayout } from '@/pages/apps/apps-layout.tsx'
import { TransactionsLayout } from '@/pages/transactions/transactions-layout.tsx'
import { SettingsLayout } from '@/pages/settings/settings-layout.tsx'
import { SftDetailLayout } from '@/pages/mint/sftDetail/sft-detail-layout.tsx'
import { TransactionsBrowserLayout } from '@/pages/txblock/transactions-browser-layout.tsx'
import { SftDetailLayoutForSelfStaking } from '@/pages/mint/sftDetailForSelfStaking/sft-detail-layout-for-self-staking.tsx'

export type NavLink = {
  icon: LucideIcon
  label: string
  path: string
  auth?: boolean
  disabled?: boolean
  element?: React.ReactElement
}

export type NavItemsType = NavLink[]

export const navItems = (): NavItemsType => {
  return [
    { icon: Bitcoin, label: 'Sidebar.assets', path: '/', auth: true, element: <AssetsLayout /> },
    { icon: Scroll, label: 'Sidebar.mint', path: '/mint', auth: true, element: <MintLayout /> },
    { icon: LayoutGrid, label: 'Sidebar.apps', path: '/apps', element: <AppsLayout /> },
    {
      icon: Compass,
      label: 'Sidebar.transactions',
      path: '/transactions',
      element: <TransactionsLayout />,
    },
    {
      icon: UserCog,
      label: 'Sidebar.settings',
      path: '/settings',
      auth: true,
      element: <SettingsLayout />,
    },
  ]
}

const otherRouter = [
  {
    path: 'mint/sft/:sftid',
    element: <SftDetailLayout />,
  },
  {
    path: '/transactions/txblock/:hash',
    element: <TransactionsBrowserLayout />,
  },
  {
    path: '/mint/sft/self-staking/:sftId',
    element: <SftDetailLayoutForSelfStaking />,
  },
]

export const routers = (): React.ReactElement => {
  return (
    <Routes>
      {navItems()
        .filter((item) => item.element != undefined || !item.disabled)
        .map((item) => <Route key={item.path} path={item.path} element={item.element} />)
        .concat(
          otherRouter.map((or) => <Route key={or.path} path={or.path} element={or.element} />),
        )}
    </Routes>
  )
}
