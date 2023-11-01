// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { Fragment, SyntheticEvent, useState } from 'react'

// ** Next Import
import { useRouter } from 'next/router'

// ** MUI Imports
import Box from '@mui/material/Box'
import Typography from '@mui/material/Typography'
import Button from '@mui/material/Button'

// ** Icon Imports

// ** Context
import { useAuth } from 'src/hooks/useAuth'
import { useSession } from 'src/hooks/useSessionAccount'

// ** Type Imports
import { Settings } from 'src/@core/context/settingsContext'
import { formatAddress } from '../../../utils/format'
import { AccountType } from '../../../../context/types'
import MenuItem from '@mui/material/MenuItem'
import Icon from '../../../components/icon'
import Divider from '@mui/material/Divider'
import Menu from '@mui/material/Menu'

interface Props {
  settings: Settings
}

const UserDropdown = (props: Props) => {
  // ** Props
  const { settings } = props

  // ** States
  const [anchorEl, setAnchorEl] = useState<Element | null>(null)

  // ** Hooks
  const router = useRouter()
  const { logout, accounts, defaultAccount } = useAuth()
  const { close } = useSession()

  // ** Vars
  const { direction } = settings

  const handleDropdownOpen = (event: SyntheticEvent) => {
    setAnchorEl(event.currentTarget)
  }

  const handleDropdownClose = (url?: string) => {
    if (url) {
      router.push(url)
    }
    setAnchorEl(null)
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

  const handleLogout = () => {
    close()
    logout()
    handleDropdownClose()
  }

  return (
    <Fragment>
      <Button variant="text" size="small" onClick={handleDropdownOpen}>
        <Box sx={{ mr: 0, display: 'flex', flexDirection: 'column', textAlign: 'center' }}>
          <Typography sx={{ fontWeight: 500 }}>{defaultAccount?.type}</Typography>
          <Typography variant="body2" sx={{ mb: 0.5, color: 'text.disabled' }}>
            {formatAddress(defaultAccount?.address ?? '')}
          </Typography>
        </Box>
      </Button>
      {defaultAccount?.type === AccountType.ROOCH ? (
        <Menu
          anchorEl={anchorEl}
          open={Boolean(anchorEl)}
          onClose={() => handleDropdownClose()}
          sx={{ '& .MuiMenu-paper': { width: 230, mt: 4 } }}
          anchorOrigin={{ vertical: 'bottom', horizontal: direction === 'ltr' ? 'right' : 'left' }}
          transformOrigin={{ vertical: 'top', horizontal: direction === 'ltr' ? 'right' : 'left' }}
        >
          {Array.from(accounts!).map((v) => {
            return (
              <MenuItem key={v[0]} sx={{ p: 0 }} onClick={() => handleDropdownClose()}>
                <Box sx={styles}>
                  <Icon icon="bx:user" />
                  <Box sx={{ display: 'flex', flexDirection: 'column' }}>
                    <Typography sx={{ fontWeight: 500 }}>{v[1].type}</Typography>
                    <Typography
                      variant="body2"
                      noWrap={true}
                      sx={{ mb: 0.5, color: 'text.disabled' }}
                    >
                      {formatAddress(v[0])}
                    </Typography>
                  </Box>
                </Box>
              </MenuItem>
            )
          })}
          <Divider />
          <MenuItem
            onClick={handleLogout}
            sx={{
              py: 2,
              px: 4,
              color: 'text.secondary',
              '& svg': { mr: 2, fontSize: '1.25rem', color: 'text.secondary' },
            }}
          >
            <Icon icon="bx:power-off" />
            Clean Loacl Account
          </MenuItem>
        </Menu>
      ) : (
        <></>
      )}
    </Fragment>
  )
}

export default UserDropdown
