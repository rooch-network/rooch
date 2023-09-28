// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, useRef, useMemo, useEffect } from 'react'

import { useAuth } from 'src/hooks/useAuth'

import Grid from '@mui/material/Grid'
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import CardHeader from '@mui/material/CardHeader'
import CardContent from '@mui/material/CardContent'
import Button from '@mui/material/Button'
import Snackbar from '@mui/material/Snackbar'
import {
  DataGrid,
  GridColDef,
  GridValueGetterParams,
  GridRenderCellParams,
  GridPaginationModel,
} from '@mui/x-data-grid'

// ** Store & Actions Imports
import { fetchData } from 'src/store/session'
import { useAppDispatch, useAppSelector } from 'src/store'

const columns: GridColDef[] = [
  { field: 'authentication_key', headerName: 'Authentication Key', width: 200 },
  {
    field: 'scopes',
    headerName: 'Scopes',
    width: 200,
    valueGetter: (params: GridValueGetterParams) => (params.row.scopes as Array<string>).join(', '),
  },
  {
    field: 'create_time',
    headerName: 'Create Time',
    width: 200,
    valueGetter: (params: GridValueGetterParams) => {
      return new Date(params.row.create_time).toLocaleString()
    },
  },
  {
    field: 'last_active_time',
    headerName: 'Last Active Time',
    width: 200,
    valueGetter: (params: GridValueGetterParams) => {
      return new Date(params.row.last_active_time).toLocaleString()
    },
  },
  {
    field: 'max_inactive_interval',
    headerName: 'Max Inactive Interval',
    width: 200,
    type: 'number',
  },
  {
    field: 'action',
    headerName: 'Action',
    width: 150,
    renderCell: (params: GridRenderCellParams) => (
      <Button
        variant="contained"
        color="secondary"
        onClick={() => handleRemove(params.row.authentication_key)}
      >
        Remove
      </Button>
    ),
  },
]

const handleRemove = (authentication_key: string) => {
  // Handle the remove action here
  console.log(`Remove session key with authentication_key: ${authentication_key}`)
}

const PAGE_SIZE = 5

export default function SessionKeyList() {
  const auth = useAuth()

  const mapPageToNextCursor = useRef<{ [page: number]: Uint8Array | null }>({})

  // ** State
  const [paginationModel, setPaginationModel] = useState({
    page: 0,
    pageSize: PAGE_SIZE,
  })

  const queryOptions = useMemo(
    () => ({
      cursor: mapPageToNextCursor.current[paginationModel.page - 1],
      pageSize: paginationModel.pageSize,
    }),
    [paginationModel],
  )

  // ** Hooks
  const dispatch = useAppDispatch()
  const { result, status, error } = useAppSelector((state) => state.session)

  useEffect(() => {
    const defaultAccount = auth.defaultAccount()
    if (!defaultAccount) {
      return
    }

    // Ignore part of request
    if (status === 'finished' || status === 'error' || status === 'loading') {
      return
    }

    dispatch(
      fetchData({
        cursor: queryOptions.cursor,
        limit: queryOptions.pageSize,
        account_address: defaultAccount.address,
        dispatch,
      }),
    )
  }, [dispatch, auth, paginationModel, result, status, queryOptions])

  useEffect(() => {
    if (status !== 'loading' && result.nextCursor) {
      // We add nextCursor when available
      mapPageToNextCursor.current[paginationModel.page] = result.nextCursor
    }
  }, [paginationModel.page, status, result.nextCursor])

  const handlePaginationModelChange = (newPaginationModel: GridPaginationModel) => {
    // We have the cursor, we can allow the page transition.
    if (newPaginationModel.page === 0 || mapPageToNextCursor.current[newPaginationModel.page - 1]) {
      setPaginationModel(newPaginationModel)
    }
  }

  const handleRefresh = () => {
    const defaultAccount = auth.defaultAccount()
    if (!defaultAccount) {
      return
    }

    dispatch(
      fetchData({
        cursor: queryOptions.cursor,
        limit: queryOptions.pageSize,
        account_address: defaultAccount.address,
        dispatch,
      }),
    )
  }

  return (
    <Grid item xs={12}>
      <Card>
        <CardHeader title="Session Keys" />
        <CardContent>
          <Box sx={{ textAlign: 'right', marginBottom: '10px' }}>
            <Button
              variant="contained"
              color="primary"
              size="small"
              onClick={() => handleRefresh()}
            >
              Refresh
            </Button>
          </Box>
          <DataGrid
            rows={status === 'finished' ? result.data : []}
            loading={status === ('loading' as 'loading')}
            columns={columns}
            checkboxSelection
            pageSizeOptions={[10, 25, 50]}
            onPaginationModelChange={handlePaginationModelChange}
            paginationModel={paginationModel}
            autoHeight
          />
          <Snackbar
            open={!!error}
            autoHideDuration={6000}
            message={error}
            anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
            action={
              <Button color="secondary" size="small" onClick={() => handleRefresh()}>
                Retry
              </Button>
            }
          />
        </CardContent>
      </Card>
    </Grid>
  )
}
