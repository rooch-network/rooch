// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React from 'react'

import './lib/i18n.ts'
import ReactDOM from 'react-dom/client'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'

import App from './App.tsx'
import ErrorPage from './components/error-page.tsx'
import { AppsLayout } from './pages/apps/apps-layout.tsx'
import { MintLayout } from './pages/mint/mint-layout.tsx'
import { SettingsLayout } from './pages/settings/settings-layout.tsx'
import { TransactionsBrowserLayout } from './pages/txblock/transactions-browser-layout.tsx'
import { TransactionsLayout } from './pages/transactions/transactions-layout.tsx'

import './styles/globals.css'

const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    errorElement: <ErrorPage />,
    children: [
      { path: '/mint', element: <MintLayout /> },
      { path: '/mint/sft/:sftId', element: <App /> },
      { path: '/mint/sft/self-staking/:sftId', element: <App /> },
      { path: '/apps', element: <AppsLayout /> },
      { path: '/transactions', element: <TransactionsLayout /> },
      { path: '/transactions/txblock/:hash', element: <TransactionsBrowserLayout /> },
      { path: '/settings', element: <SettingsLayout /> },
    ],
  },
  {
    path: '*',
    element: <ErrorPage />,
  },
])

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
)
