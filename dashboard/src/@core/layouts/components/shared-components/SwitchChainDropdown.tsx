// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import {useState, SyntheticEvent, Fragment} from 'react'

// ** MUI Imports
import Box from '@mui/material/Box'
import Menu from '@mui/material/Menu'
import Divider from '@mui/material/Divider'
import MenuItem from '@mui/material/MenuItem'
import Typography from '@mui/material/Typography'
import Button from '@mui/material/Button'

// ** Type Imports
import {Settings} from 'src/@core/context/settingsContext'
import Icon from "../../../components/icon";

interface Props {
    settings: Settings
}

const chainIDData = ['dev', 'test']

const SwitchChainDropdown = (props: Props) => {
    // ** Props
    const {settings} = props

    // ** States
    const [anchorEl, setAnchorEl] = useState<Element | null>(null)
    const [chainID, setChainId] = useState<string>('dev')

    // ** Vars
    const {direction} = settings

    const handleDropdownOpen = (event: SyntheticEvent) => {
        setAnchorEl(event.currentTarget)
    }

    const handleDropdownClose = () => {
        setAnchorEl(null)
    }

    return (
        <Fragment>
            <Button variant="text" size="small" onClick={handleDropdownOpen}>
                <Box sx={{mr: 0, display: 'flex', flexDirection: 'column', textAlign: 'center'}}>
                    <Typography sx={{fontWeight: 500}}>{chainID}</Typography>
                </Box>
            </Button>
            <Menu
                anchorEl={anchorEl}
                open={Boolean(anchorEl)}
                onClose={() => handleDropdownClose()}
                sx={{'& .MuiMenu-paper': {width: 120, mt: 4}}}
                anchorOrigin={{vertical: 'bottom', horizontal: direction === 'ltr' ? 'right' : 'left'}}
                transformOrigin={{vertical: 'top', horizontal: direction === 'ltr' ? 'right' : 'left'}}
            >
                {chainIDData.map((v, i) => (
                    <MenuItem
                        key={v}
                        onClick={() => setChainId(v)}
                        sx={{
                            color: v === chainID ? 'text.primary' : 'text.secondary',
                            '& svg': {mr: 2, fontSize: '1.25rem', color: 'text.secondary'},
                            display: 'flex',
                            justifyContent: 'center',
                        }}
                    >
                        {v.toUpperCase()}
                    </MenuItem>
                ))}
            </Menu>
        </Fragment>
    )
}

export default SwitchChainDropdown
