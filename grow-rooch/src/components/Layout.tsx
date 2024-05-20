import React from 'react'

import { Footer } from './Footer'
import { Header } from './Header'

export const Layout = ({ children }: { children: React.ReactNode }) => {
  return (
    <>
      <div className="flex flex-col min-h-screen">
        <Header />
        <main className="flex-auto mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">{children}</main>
        <Footer />
      </div>
    </>
  )
}
