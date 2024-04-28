// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { WalletConnect } from './wallet-connect'

export const NavbarItem = () => {
  return (
    <div className="flex items-center justify-end">
      <WalletConnect />
    </div>
  )
}
