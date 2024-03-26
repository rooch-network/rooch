// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** MUI Imports
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'

// ** Type Imports
import { Settings } from 'src/@core/context/settingsContext'

import Typography from '@mui/material/Typography'
import { AllNetwork, Network } from '@roochnetwork/rooch-sdk'
import Menu from '@mui/material/Menu'
import { useRoochClientContext } from '@roochnetwork/rooch-sdk-kit'
import MenuItem from '@mui/material/MenuItem'
import { Fragment, SyntheticEvent, useState } from 'react'

interface Props {
  settings: Settings
}

const styles = {
  py: 2,
  px: 4,
  width: '100%',
  display: 'flex',
  alignItems: 'center',
  color: 'text.secondary',
  textDecoration: 'none',
  '& svg': {
    mr: 2,
    fontSize: '1.25rem',
    color: 'text.secondary',
  },
}

const SwitchNetworkDropdown = (props: Props) => {
  const { settings } = props

  // const chain = useWalletStore(state => state.currentChain)
  // const setChain = useWalletStore(state => state.setChain)
  const { currentNetwork, selectedNetwork } = useRoochClientContext()

  // ** States
  const [anchorEl, setAnchorEl] = useState<Element | null>(null)

  // ** Vars
  const { direction } = settings

  const handleDropdownOpen = (event: SyntheticEvent) => {
    setAnchorEl(event.currentTarget)
  }

  const handleDropdownClose = (v?: Network) => {
    setAnchorEl(null)
    if (v) {
      selectedNetwork(v)
    }
  }

  return (
    <Fragment>
      <Button variant="text" size="small" onClick={handleDropdownOpen}>
        <Box sx={{ mr: 0, display: 'flex', flexDirection: 'column', textAlign: 'center' }}>
          <Typography sx={{ fontWeight: 500 }}>{currentNetwork.name}</Typography>
        </Box>
      </Button>
      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={() => handleDropdownClose()}
        sx={{ '& .MuiMenu-paper': { width: 100, mt: 4 } }}
        anchorOrigin={{ vertical: 'bottom', horizontal: direction === 'ltr' ? 'right' : 'left' }}
        transformOrigin={{ vertical: 'top', horizontal: direction === 'ltr' ? 'right' : 'left' }}
      >
        {AllNetwork.map((v: Network) => {
          return (
            <MenuItem key={v.id} sx={{ p: 0 }} onClick={() => handleDropdownClose(v)}>
              <Box sx={styles}>
                <Box sx={{ display: 'flex', flexDirection: 'column' }}>
                  <Typography sx={{ fontWeight: 500 }}>{v.name.toUpperCase()}</Typography>
                </Box>
              </Box>
            </MenuItem>
          )
        })}
      </Menu>
    </Fragment>
  )
}

export default SwitchNetworkDropdown
