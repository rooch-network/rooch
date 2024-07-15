// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React from 'react'

import './lib/i18n.ts'
import ReactDOM from 'react-dom/client'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'

import App from './App.tsx'
import ErrorPage from './components/error-page.tsx'
import './styles/globals.css'
import { allRouter } from '@/navigation'

const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    errorElement: <ErrorPage />,
    children: allRouter,
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
