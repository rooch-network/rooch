// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState } from 'react'

import { useConnectWallet, useWallets, useWalletStore } from '@roochnetwork/rooch-sdk-kit'

// ** MUI Imports
import Button from '@mui/material/Button'

// ** Type Imports
import { Settings } from 'src/@core/context/settingsContext'
import { formatAddress } from '../../../utils/format'

interface Props {
  settings: Settings
}

const UserDropdown = (_: Props) => {
  // ** States
  const [loading, setLoading] = useState(false)

  // ** Hooks
  const account = useWalletStore((state) => state.currentAccount)
  const { mutateAsync: connectWallet } = useConnectWallet()
  const wallets = useWallets().filter((w) => w.installed)

  const handleConnect = async () => {
    setLoading(true)
    if (account === null && wallets.length > 0) {
      await connectWallet({ wallet: wallets[0] })
    }

    setLoading(false)
  }

  return (
    <Button
      disabled={loading}
      onClick={handleConnect}
      sx={{
        py: 2,
        px: 4,
        color: 'theme.palette.primary.main',
      }}
    >
      {account === null ? 'connect' : formatAddress(account?.address)}
    </Button>
  )
}

export default UserDropdown
