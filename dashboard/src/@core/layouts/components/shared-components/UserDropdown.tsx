// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState } from 'react'

import { useConnectWallet, useWalletStore } from '@roochnetwork/rooch-sdk-kit'

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

  const handleConnect = async () => {
    setLoading(true)
    if (account === null) {
      await connectWallet()
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
      {account === null ? 'connect' : formatAddress(account?.getAddress())}
    </Button>
  )
}

export default UserDropdown
