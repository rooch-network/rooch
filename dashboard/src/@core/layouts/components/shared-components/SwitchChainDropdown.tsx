// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, SyntheticEvent, Fragment } from 'react'

// ** MUI Imports
import Box from '@mui/material/Box'
import Menu from '@mui/material/Menu'
import MenuItem from '@mui/material/MenuItem'
import Typography from '@mui/material/Typography'
import Button from '@mui/material/Button'

// ** Type Imports
import { Settings } from 'src/@core/context/settingsContext'

// ** Hooks

import { useRooch } from 'src/hooks/useRooch'
import { Chain } from '@rooch/sdk'
import { ca } from 'date-fns/locale'

interface Props {
  settings: Settings
}

const SwitchChainDropdown = (props: Props) => {
  // ** Props
  const { settings } = props

  // ** Hooks
  const rooch = useRooch()

  // ** States
  const [anchorEl, setAnchorEl] = useState<Element | null>(null)
  const [chain, setChain] = useState<Chain>(rooch.getActiveChina())

  // ** Vars
  const { direction } = settings

  const handleDropdownOpen = (event: SyntheticEvent) => {
    setAnchorEl(event.currentTarget)
  }

  const handleDropdownClose = () => {
    setAnchorEl(null)
  }

  const handleSwitchChain = async (chain: Chain) => {
    await rooch.switchChina(chain)
    setChain(chain)
    window.location.reload()
  }

  return (
    <Fragment>
      <Button variant="text" size="small" onClick={handleDropdownOpen}>
        <Box sx={{ mr: 0, display: 'flex', flexDirection: 'column', textAlign: 'center' }}>
          <Typography sx={{ fontWeight: 500 }}>{chain.name}</Typography>
        </Box>
      </Button>
      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={() => handleDropdownClose()}
        sx={{ '& .MuiMenu-paper': { width: 120, mt: 4 } }}
        anchorOrigin={{ vertical: 'bottom', horizontal: direction === 'ltr' ? 'right' : 'left' }}
        transformOrigin={{ vertical: 'top', horizontal: direction === 'ltr' ? 'right' : 'left' }}
      >
        {rooch.getAllChina().map((v, i) => (
          <MenuItem
            key={v.id}
            onClick={() => handleSwitchChain(v)}
            sx={{
              color: v === chain ? 'text.primary' : 'text.secondary',
              '& svg': { mr: 2, fontSize: '1.25rem', color: 'text.secondary' },
              display: 'flex',
              justifyContent: 'center',
            }}
          >
            {v.name.toUpperCase()}
          </MenuItem>
        ))}
      </Menu>
    </Fragment>
  )
}

export default SwitchChainDropdown
