// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, useEffect, useCallback } from 'react'

// ** MUI Imports
import Grid from '@mui/material/Grid'
import Card from '@mui/material/Card'
import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
import CardHeader from '@mui/material/CardHeader'
import Typography from '@mui/material/Typography'
import CardContent from '@mui/material/CardContent'
import InputAdornment from '@mui/material/InputAdornment'

// ** Store & Actions Imports
import { fetchData } from 'src/store/scan/state/get'
import { useAppDispatch, useAppSelector } from 'src/store'

// ** SDK Imports
import Icon from 'src/@core/components/icon'
import CardSnippet from 'src/@core/components/card-snippet'
import { useRooch } from '../../../../hooks/useRooch'

const StateGetView = () => {
  const rooch = useRooch()

  // ** State
  const [accessPath, setAccessPath] = useState<string>('/object/0x1')

  // ** Hooks
  const dispatch = useAppDispatch()
  const { result, status, error } = useAppSelector((state) => state.stateView)

  const handleSearch = () => {
    dispatch(fetchData({ provider: rooch.provider!, dispatch, accessPath }))
  }

  useEffect(() => {
    handleSearch()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dispatch])

  // Handle shortcut keys keyup events
  const handleKeyUp = useCallback(
    (event: KeyboardEvent) => {
      if (event.keyCode === 13) {
        dispatch(fetchData({ provider: rooch.provider!, dispatch, accessPath }))
      }
    },
    [dispatch, accessPath, rooch],
  )

  useEffect(() => {
    document.addEventListener('keyup', handleKeyUp)

    return () => {
      document.removeEventListener('keyup', handleKeyUp)
    }
  }, [handleKeyUp])

  return (
    <Grid container spacing={6}>
      <Grid item xs={12}>
        <Card>
          <CardHeader title="State Filters" />
          <CardContent>
            <TextField
              id="access-path-id"
              label="Access Path"
              fullWidth
              value={accessPath}
              helperText={error?.toString()}
              InputProps={{
                endAdornment: (
                  <InputAdornment
                    position="end"
                    sx={{ color: 'text.primary' }}
                    onClick={handleSearch}
                  >
                    <Button size="small">
                      <Typography mr={2} color="text.disabled">
                        Enter
                      </Typography>
                      <Icon icon="bx:search" />
                    </Button>
                  </InputAdornment>
                ),
              }}
              onChange={(v) => setAccessPath(v.target.value)}
            />
          </CardContent>
        </Card>
      </Grid>

      <Grid item xs={12}>
        {status === 'error' ? null : (
          <CardSnippet
            defaultShow={true}
            fullHeight={true}
            codes={[
              {
                code: JSON.stringify(result, null, 2),
                lng: 'json',
              },
            ]}
          />
        )}
      </Grid>
    </Grid>
  )
}

export default StateGetView
