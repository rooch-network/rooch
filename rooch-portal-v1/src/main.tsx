import React from 'react'
import './lib/i18n.ts'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './styles/globals.css'
import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom'
import ErrorPage from './components/error-page.tsx'

const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    errorElement: <ErrorPage />,
    children: [
      { path: '/', element: <App /> },
      { path: '/mint', element: <App /> },
      { path: '/mint/sft/:sftId', element: <App /> },
      { path: '/mint/sft/self-staking/:sftId', element: <App /> },
      { path: '/apps', element: <App /> },
      { path: '/transactions', element: <App /> },
      { path: '/settings', element: <App /> },
    ],
  },
  {
    path: '*',
    element: <Navigate to="/" replace />,
  },
])

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
)
