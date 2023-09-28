// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import React, { useState, SyntheticEvent, Fragment } from 'react'

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
import { useETH } from 'src/hooks/useETH'

// ** SDK
import { Chain } from '@rooch/sdk'
import { Dialog, DialogActions, DialogContent, DialogTitle, TextField } from '@mui/material'
import { Controller, useForm } from 'react-hook-form'
import { yupResolver } from '@hookform/resolvers/yup'
import * as yup from 'yup'
import FormHelperText from '@mui/material/FormHelperText'
import FormControl from '@mui/material/FormControl'
import toast from 'react-hot-toast'

interface Props {
  settings: Settings
}

const schema = yup.object().shape({
  chainId: yup.number().min(1).required(),
  rpc: yup.string().url().required(),
})

const defaultValues = {
  chainId: 1,
  name: '',
  rpc: '',
}

interface FormData {
  chainId: number
  name: string
  rpc: string
}

const SwitchChainDropdown = (props: Props) => {
  // ** Props
  const { settings } = props

  // ** Hooks
  const eth = useETH()
  const rooch = useRooch()
  const {
    control,

    setError,
    handleSubmit,
    formState: { errors },
  } = useForm({
    defaultValues,
    mode: 'onSubmit',
    resolver: yupResolver(schema),
  })

  // ** States
  const [anchorEl, setAnchorEl] = useState<Element | null>(null)
  const [chain, setChain] = useState<Chain>(rooch.getActiveChina())
  const [show, setShow] = useState(false)
  const [loading, setLoading] = useState(false)

  // ** Vars
  const { direction } = settings

  const handleDropdownOpen = (event: SyntheticEvent) => {
    setAnchorEl(event.currentTarget)
  }

  const handleDropdownClose = () => {
    setLoading(false)
    setAnchorEl(null)
  }

  const handleSwitchChain = async (chain: Chain) => {
    if (eth.isConnect) {
      await eth.switchChina(chain.info)
    }

    await rooch.switchChina(chain)

    setChain(chain)
    window.location.reload()
  }

  const handleCustom = () => {
    handleDropdownClose()
    setShow(true)
  }

  const onSubmit = async (data?: FormData) => {
    setLoading(true)

    if (!data) {
      return
    }

    if (!data.rpc.startsWith('https')) {
      setError('rpc', {
        type: 'manual',
        message: 'Expected https',
      })

      return
    }

    if (rooch.getAllChina().find((v) => v.id === data.chainId && v.url === data.rpc)) {
      toast.error('The chain already exists', {
        duration: 1000,
      })
      setLoading(false)

      return
    }

    const chain = new Chain(data.chainId, data.name, {
      url: data.rpc,
    })

    try {
      if (eth.isConnect) {
        await eth.addChina(chain.info)
      }

      await rooch.addChina(chain)
    } catch (e) {
      setLoading(false)
    }

    setLoading(false)
  }

  return (
    <>
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
          sx={{ '& .MuiMenu-paper': { width: 160, mt: 4 } }}
          anchorOrigin={{ vertical: 'bottom', horizontal: direction === 'ltr' ? 'right' : 'left' }}
          transformOrigin={{ vertical: 'top', horizontal: direction === 'ltr' ? 'right' : 'left' }}
        >
          {rooch.getAllChina().map((v) => (
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
          <MenuItem
            key="new"
            onClick={handleCustom}
            sx={{
              color: 'text.secondary',
              '& svg': { mr: 2, fontSize: '1.25rem', color: 'text.secondary' },
              display: 'flex',
              justifyContent: 'center',
            }}
          >
            {'custom'.toUpperCase()}
          </MenuItem>
        </Menu>
      </Fragment>
      <Dialog open={show}>
        <DialogTitle>Add custom chain</DialogTitle>
        <form noValidate autoComplete="off" onSubmit={handleSubmit(onSubmit)}>
          <DialogContent>
            <FormControl fullWidth sx={{ mb: 4 }}>
              <Controller
                name="chainId"
                control={control}
                rules={{ required: true }}
                render={({ field: { value, onChange, onBlur } }) => (
                  <TextField
                    autoFocus
                    margin="dense"
                    label="Chain Id"
                    value={value}
                    onBlur={onBlur}
                    type="number"
                    variant="standard"
                    fullWidth
                    onChange={onChange}
                    error={Boolean(errors.chainId)}
                    placeholder=""
                  />
                )}
              />
              {errors.chainId && (
                <FormHelperText sx={{ color: 'error.main' }}>
                  {errors.chainId.message}
                </FormHelperText>
              )}
            </FormControl>

            <FormControl fullWidth sx={{ mb: 4 }}>
              <Controller
                name="name"
                control={control}
                rules={{ required: true }}
                render={({ field: { value, onChange, onBlur } }) => (
                  <TextField
                    margin="dense"
                    label="Name"
                    value={value}
                    onBlur={onBlur}
                    variant="standard"
                    fullWidth
                    onChange={onChange}
                    error={Boolean(errors.name)}
                    placeholder=""
                  />
                )}
              />
              {errors.name && (
                <FormHelperText sx={{ color: 'error.main' }}>{errors.name.message}</FormHelperText>
              )}
            </FormControl>
            <FormControl fullWidth sx={{ mb: 4 }}>
              <Controller
                name="rpc"
                control={control}
                rules={{ required: true }}
                render={({ field: { value, onChange, onBlur } }) => (
                  <TextField
                    margin="dense"
                    label="RPC"
                    value={value}
                    onBlur={onBlur}
                    variant="standard"
                    fullWidth
                    onChange={onChange}
                    error={Boolean(errors.rpc)}
                    placeholder=""
                  />
                )}
              />
              {errors.rpc && (
                <FormHelperText sx={{ color: 'error.main' }}>{errors.rpc.message}</FormHelperText>
              )}
            </FormControl>
          </DialogContent>
          <DialogActions>
            <Button onClick={() => setShow(false)}>Cancel</Button>
            <Button type="submit" disabled={loading}>
              Add
            </Button>
          </DialogActions>
        </form>
      </Dialog>
    </>
  )
}

export default SwitchChainDropdown
